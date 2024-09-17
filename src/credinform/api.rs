use anyhow::{anyhow, Error, Result};
use reqwest::Url;

use crate::config::Client;

use super::models::{
    AccessToken, Address, CredinformData, CredinformFile, SearchCompany, TaxNumber,
};
use log::{debug, error, info, warn};

pub async fn get_token(client: &Client) -> Result<AccessToken, Error> {
    let response = client
        .post("https://restapi.credinform.ru/api/Authorization/GetAccessKey")
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&serde_json::json!({"username": client.username(), "password": client.password()}))
        .send()
        .await?;

    if response.status().is_success() {
        let response = response.json::<serde_json::Value>().await?;
        let access_key = response
            .get("accessKey")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Failed to get token, check your credentials in config.toml"))?;

        Ok(AccessToken::new(access_key))
    } else {
        Err(anyhow!(
            "Failed get token, check your credentials in config.toml"
        ))
    }
}

async fn search_company(
    client: &Client,
    access_key: &AccessToken,
    tax_number: &TaxNumber,
) -> Result<SearchCompany> {
    let url = Url::parse_with_params(
        "https://restapi.credinform.ru/api/Search/SearchCompany",
        &[("apiVersion", client.api_version())],
    )
    .unwrap();

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
        .await
        .unwrap()
        .json::<serde_json::Value>()
        .await
        .unwrap();

    let company_data_list = response
        .get("companyDataList")
        .and_then(|v| v.as_array())
        .ok_or("Failed to get companyDataList")
        .unwrap();

    let company_id = company_data_list
        .first()
        .and_then(|v| v.get("companyId").and_then(|v| v.as_str()))
        .ok_or("Failed to get companyId. Company not found")
        .unwrap();

    let company_name = company_data_list
        .first()
        .and_then(|v| v.get("companyName").and_then(|v| v.as_str()))
        .ok_or("Failed to get companyName. Company not found")
        .unwrap();

    Ok(SearchCompany::new(company_id, company_name))
}

pub async fn get_data(
    client: &Client,
    access_key: &AccessToken,
    tax_number: &TaxNumber,
    address: &Address,
) -> Result<CredinformData> {
    let url = Url::parse_with_params(
        format!(
            "https://restapi.credinform.ru/api/CompanyInformation/{}",
            address
        )
        .as_str(),
        &[("apiVersion", client.api_version())],
    )?;
    debug!("URL: {}", url);
    debug!("Tax number: {}", tax_number);

    let company = search_company(client, access_key, tax_number).await?;
    debug!("Company ID: {}", company.id);
    debug!("Company Name: {}", company.name);

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
        let response = response.json::<serde_json::Value>().await.unwrap();
        let data = CredinformData::new(company.name.as_str(), response);
        Ok(data.unwrap())
    } else {
        Err(anyhow!("Failed request to {}", url))
    }
}

pub async fn get_trademarks(
    client: &Client,
    access_key: &AccessToken,
    tax_number: &TaxNumber,
) -> Result<CredinformData> {
    let data = get_data(client, access_key, tax_number, &Address::new("Trademarks")).await?;

    match data.data.get("trademarkList") {
        Some(trademarks) => {
            let trademarks = trademarks.as_array().unwrap();
            for trademark in trademarks {
                let file_image = &trademark["fileImage"];
                let file = CredinformFile::new(data.company_name.as_str(), &file_image);
                match file {
                    Ok(file) => {
                        file.save(tax_number)?;
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                    }
                }
            }
        }
        None => return Err(anyhow!("No trademarks")),
    };

    Ok(data)
}
