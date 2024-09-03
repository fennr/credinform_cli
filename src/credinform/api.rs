use reqwest::Url;
use std::error::Error;

use crate::config::Client;

use super::models::{AccessToken, Address, CredinformData, SearchCompany, TaxNumber};

pub async fn get_token(client: &Client) -> Result<AccessToken, Box<dyn Error>> {
    let response = client
        .post("https://restapi.credinform.ru/api/Authorization/GetAccessKey")
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&serde_json::json!({"username": client.username(), "password": client.password()}))
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    let access_key = response
        .get("accessKey")
        .and_then(|v| v.as_str())
        .ok_or("Failed to get token, check your credentials in config.toml")?;

    Ok(AccessToken::new(access_key))
}

async fn search_company(
    client: &Client,
    access_key: &AccessToken,
    tax_number: &TaxNumber,
) -> Result<SearchCompany, Box<dyn Error>> {
    let url = Url::parse_with_params(
        "https://restapi.credinform.ru/api/Search/SearchCompany",
        &[("apiVersion", client.api_version())],
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
        .and_then(|v| v.get("companyId").and_then(|v| v.as_str()))
        .ok_or("Failed to get companyId. Company not found")?;

    let company_name = company_data_list
        .first()
        .and_then(|v| v.get("companyName").and_then(|v| v.as_str()))
        .ok_or("Failed to get companyName. Company not found")?;

    Ok(SearchCompany::new(company_id, company_name))
}

pub async fn get_data(
    client: &Client,
    access_key: &AccessToken,
    tax_number: &TaxNumber,
    address: &Address,
) -> Result<CredinformData, Box<dyn Error>> {
    let url = Url::parse_with_params(
        format!(
            "https://restapi.credinform.ru/api/CompanyInformation/{}",
            address
        )
        .as_str(),
        &[("apiVersion", client.api_version())],
    )?;
    println!("URL: {}", url);
    println!("Tax number: {}", tax_number);

    let company = search_company(client, access_key, tax_number).await?;
    println!("Company ID: {}", company.id);
    println!("Company Name: {}", company.name);

    let response = client
        .post(url.clone())
        .header("Content-Type", "application/json-patch+json")
        .header("Accept", "text/plain")
        .header("accessKey", access_key.to_string())
        .json(&serde_json::json!({
            "companyId": company.id,
            "language": "Russian",
        }))
        .send()
        .await?;

    if response.status().is_success() {
        let response = response.json::<serde_json::Value>().await?;
        let data = CredinformData::new(company.name.as_str(), response);
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
