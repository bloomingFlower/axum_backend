use modql::filter::ListOptions;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_with::{serde_as, OneOrMany};

#[derive(Deserialize)]
pub struct ParamsForCreate<D> {
    pub data: D,
}

#[derive(Deserialize)]
pub struct ParamsForUpdate<D> {
    pub id: i64,
    pub data: D,
}

#[derive(Deserialize)]
pub struct ParamsIded {
    pub id: i64,
}

#[serde_as]
#[derive(Deserialize)]
pub struct ParamsList<F>
where
    F: DeserializeOwned,
{
    #[serde_as(deserialize_as = "Option<OneOrMany<_>>")]
    filters: Option<Vec<F>>,
    list_options: Option<ListOptions>,
}
