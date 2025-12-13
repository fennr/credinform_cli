use anyhow::{anyhow, Context, Error, Result};
use reqwest::Url;

use crate::config::Client;

use super::models::{
    AccessToken, Address, CredinformData, CredinformFile, SearchCompany, TaxNumber,
};
use log::{debug, error, warn};

pub async fn get_token(client: &Client) -> Result<AccessToken, Error> {
    let response = client
        .post("https://restapi.credinform.ru/api/Authorization/GetAccessKey")
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&serde_json::json!({"username": client.username(), "password": client.password()}))
        .send()
        .await
        .context("Ошибка сети при получении токена credinform")?
        .error_for_status()
        .context("Credinform вернул ошибку авторизации, проверьте логин/пароль")?;

    let response = response
        .json::<serde_json::Value>()
        .await
        .context("Не удалось разобрать ответ авторизации credinform")?;
    let access_key = response
        .get("accessKey")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            anyhow!("Не удалось получить accessKey, проверьте логин/пароль в config.toml")
        })?;

    Ok(AccessToken::new(access_key))
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
    .context("Некорректный адрес поиска компании")?;

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
        .with_context(|| format!("Ошибка сети при поиске компании с ИНН {}", tax_number))?
        .error_for_status()
        .context("Credinform вернул ошибку при поиске компании")?;

    let response = response
        .json::<serde_json::Value>()
        .await
        .context("Не удалось распарсить ответ поиска компании")?;

    let company_data_list = response
        .get("companyDataList")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow!("Компания с ИНН {} не найдена", tax_number))?;

    let company_id = company_data_list
        .first()
        .and_then(|v| v.get("companyId").and_then(|v| v.as_str()))
        .ok_or_else(|| anyhow!("Не удалось получить companyId для {}", tax_number))?;

    let company_name = company_data_list
        .first()
        .and_then(|v| v.get("companyName").and_then(|v| v.as_str()))
        .ok_or_else(|| anyhow!("Не удалось получить companyName для {}", tax_number))?;

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
    )
    .with_context(|| format!("Некорректный адрес ручки {}", address))?;
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
        .await
        .with_context(|| format!("Ошибка сети при запросе {} для {}", url, tax_number))?
        .error_for_status()
        .with_context(|| format!("Credinform вернул ошибку по адресу {}", address))?;

    let response = response
        .json::<serde_json::Value>()
        .await
        .context("Не удалось прочитать тело ответа credinform")?;
    let data = CredinformData::new(company.name.as_str(), response)
        .map_err(|e| anyhow!("Не удалось собрать данные по {}: {}", address, e))?;
    Ok(data)
}

pub async fn get_trademarks(
    client: &Client,
    access_key: &AccessToken,
    tax_number: &TaxNumber,
) -> Result<CredinformData> {
    let data = get_data(client, access_key, tax_number, &Address::new("Trademarks")).await?;

    if let Some(trademarks) = data.data.get("trademarkList").and_then(|v| v.as_array()) {
        for trademark in trademarks {
            let file_image = &trademark["fileImage"];
            let file = CredinformFile::new(data.company_name.as_str(), file_image);
            match file {
                Ok(file) => {
                    file.save(tax_number)?;
                }
                Err(e) => {
                    error!("Error: {}", e);
                }
            }
        }
    } else {
        warn!("No trademarks found");
    }

    Ok(data)
}
