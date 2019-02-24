use std::fmt::Debug;
use std::ops::Add;
use std::str::FromStr;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use biscuit::jwa::SignatureAlgorithm;
use biscuit::jws::{Header, RegisteredHeader, Secret};
use biscuit::{ClaimsSet, Empty, RegisteredClaims, JWT};
use futures::future::{self, Either, IntoFuture};
use futures::Future;
use reqwest::Client as HttpClient;
use ring::signature::{ECDSAKeyPair, ECDSA_P256_SHA256_FIXED_SIGNING};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use untrusted::Input;

use crate::errors::{Error, ErrorKind, Result};

const TOKEN_DURATION_IN_SECS: u64 = 3600;
const URL_ROOT_PATH: &str = "https://api.music.apple.com/v1/";

#[derive(Debug, Deserialize)]
pub struct ResponseErrorSource {
    parameter: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ResponseError {
    code: String,
    detail: Option<String>,
    id: String,
    source: Option<ResponseErrorSource>,
    status: String,
    title: String,
}

#[derive(Debug, Deserialize)]
pub struct Response<T> {
    data: Option<T>,
    errors: Option<Vec<ResponseError>>,
    href: Option<String>,
    next: Option<String>,
    result: Option<T>,
}

impl<T> Response<T> {
    pub fn data(&self) -> Option<&T> {
        self.data.as_ref()
    }

    pub fn result(&self) -> Option<&T> {
        self.result.as_ref()
    }
}

#[derive(Debug, Deserialize)]
pub struct SearchResults {
    songs: Option<Vec<Response<Song>>>,
}

#[derive(Debug, Deserialize)]
pub struct Song {
    #[serde(rename = "type")]
    type_: String,
    attributes: Option<SongAttributes>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SongAttributes {
    album_name: String,
    artist_name: String,
    composer_name: Option<String>,
    content_rating: Option<String>,
    disc_number: String,
    duration_in_millis: Option<String>,
    genre_names: Vec<String>,
    isrc: String,
    name: String,
    url: String,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
struct KeyInfo {
    #[serde(rename = "kid")]
    key_id: String,
}

pub struct Client {
    team_id: String,
    key_id: String,
    key_secret_pem: String,
    storefront: String,
    http_client: HttpClient,
}

impl Client {
    pub fn new(
        team_id: String,
        key_id: String,
        key_secret_pem: String,
        storefront: String,
    ) -> Self {
        Client {
            team_id,
            key_id,
            key_secret_pem,
            storefront,
            http_client: HttpClient::new(),
        }
    }

    pub fn search(
        &self,
        terms: &[&str],
    ) -> impl Future<Item = Response<SearchResults>, Error = Error> {
        self.generate_token()
            .and_then(|token| {
                let url = format!(
                    "{url_base}catalog/{storefront}/search",
                    url_base = URL_ROOT_PATH,
                    storefront = self.storefront
                );
                self.http_client
                    .get(&url)
                    .query(&[
                        ("term", terms.join("+").replace(" ", "+")),
                        ("limit", "1".to_owned()),
                        ("types", "songs".to_owned()),
                    ])
                    .header("Authorization", format!("Bearer {token}", token = token))
                    .send()
                    .map_err(From::from)
                    .and_then(|mut res| {
                        if !res.status().is_success() {
                            Err(Error::from(ErrorKind::Service(res.status().as_u16())))
                        } else {
                            Ok(res.json().map_err(Error::from))
                        }
                    })
            })
            .into_future()
            .flatten()
    }

    pub fn generate_token(&self) -> Result<String> {
        let claims = ClaimsSet::<Empty> {
            registered: RegisteredClaims {
                issuer: Some(FromStr::from_str(&self.team_id)?),
                expiry: Some(From::from(
                    SystemTime::now()
                        .add(Duration::from_secs(TOKEN_DURATION_IN_SECS))
                        .duration_since(UNIX_EPOCH)?
                        .as_secs() as i64,
                )),
                issued_at: Some(From::from(
                    SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64,
                )),
                ..Default::default()
            },
            private: Default::default(),
        };

        let token = JWT::new_decoded(
            Header {
                registered: RegisteredHeader {
                    algorithm: SignatureAlgorithm::ES256,
                    ..Default::default()
                },
                private: KeyInfo {
                    key_id: self.key_id.clone(),
                },
            },
            claims,
        );

        let secret = pem::parse(&self.key_secret_pem)?;
        let key_pair = ECDSAKeyPair::from_pkcs8(
            &ECDSA_P256_SHA256_FIXED_SIGNING,
            Input::from(&secret.contents),
        )?;

        let secret = Secret::ECDSAKeyPair(Arc::new(key_pair));
        let signed_token = token.into_encoded(&secret)?;

        Ok(signed_token.unwrap_encoded().to_string())
    }
}
