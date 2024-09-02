use dotenv::dotenv;
use reqwest::{Client, Url};
use std::error::Error;

use super::models::{AccessToken, Address, CredinformData, TaxNumber};

pub async fn get_token(client: &Client) -> Result<AccessToken, Box<dyn Error>> {
    dotenv().ok();

    let username = std::env::var("CREDINFORM_USERNAME").map_err(|_| {
        Box::<dyn std::error::Error>::from("Failed to get CREDINFORM_USERNAME environment variable")
    })?;
    let password = std::env::var("CREDINFORM_PASSWORD").map_err(|_| {
        Box::<dyn std::error::Error>::from("Failed to get CREDINFORM_PASSWORD environment variable")
    })?;

    let response = client
        .post("https://restapi.credinform.ru/api/Authorization/GetAccessKey")
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&serde_json::json!({"username": username, "password": password}))
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    let access_key = response
        .get("accessKey")
        .and_then(|v| v.as_str())
        .ok_or("Failed to get token")?;

    Ok(AccessToken::new(access_key))
}

async fn search_company(
    client: &Client,
    access_key: &AccessToken,
    tax_number: &TaxNumber,
) -> Result<String, Box<dyn Error>> {
    dotenv().ok();
    let api_version = std::env::var("CREDINFORM_API_VERSION")?;

    let url = Url::parse_with_params(
        "https://restapi.credinform.ru/api/Search/SearchCompany",
        &[("apiVersion", api_version.as_str())],
    )?;

    let response = client
        .post(url)
        .header("Content-Type", "application/json-patch+json")
        .header("Accept", "text/plain")
        .header("accessKey", access_key.to_string())
        .json(&serde_json::json!({
            "language": "Russian",
            "searchCompanyParameters": {
                "taxNumber": tax_number
            }
        }))
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    let company_data_list = response
        .get("companyDataList")
        .and_then(|v| v.as_array())
        .ok_or("Failed to get companyDataList")?;

    let company_id = company_data_list
        .first()
        .and_then(|v| v.get("companyId"))
        .and_then(|v| v.as_str())
        .ok_or("Failed to get companyId. Company not found")?;

    Ok(company_id.to_string())
}

pub async fn get_data(
    client: &Client,
    access_key: &AccessToken,
    tax_number: &TaxNumber,
    address: &Address,
) -> Result<CredinformData, Box<dyn Error>> {
    dotenv().ok();
    let api_version = std::env::var("CREDINFORM_API_VERSION")?;

    let url = Url::parse_with_params(
        format!(
            "https://restapi.credinform.ru/api/CompanyInformation/{}",
            address
        )
        .as_str(),
        &[("apiVersion", api_version.as_str())],
    )?;
    println!("URL: {}", url);
    println!("Tax number: {}", tax_number);

    let company_id = search_company(client, access_key, tax_number).await?;
    println!("Company ID: {}", company_id);

    let response = client
        .post(url.clone())
        .header("Content-Type", "application/json-patch+json")
        .header("Accept", "text/plain")
        .header("accessKey", access_key.to_string())
        .json(&serde_json::json!({
            "companyId": company_id,
            "language": "Russian",
        }))
        .send()
        .await?;

    if response.status().is_success() {
        let response = response.json::<serde_json::Value>().await?;
        let data = CredinformData::new(response);
        println!("Data: {:?}", data);
        Ok(data?)
    } else {
        Err(format!(
            "Failed request to {}; Request status: {})",
            url,
            response.status()
        )
        .into())
    }
}
