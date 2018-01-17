use std::io::Error;

use grt::resource::ResourceBuilder;
use grt::util::invalid_data_error;
use grt::serde_json;
use grt::serde_yaml;

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Rules {
    pub base_ap: u32,
    pub movement_ap: u32,
    pub base_initiative: u32,
}

impl ResourceBuilder for Rules {
    fn owned_id(&self) -> String {
        "Rules".to_string()
    }

    fn from_json(data: &str) -> Result<Rules, Error> {
        let rules: Rules = serde_json::from_str(data)?;

        Ok(rules)
    }

    fn from_yaml(data: &str) -> Result<Rules, Error> {
        let resource: Result<Rules, serde_yaml::Error> = serde_yaml::from_str(data);

        match resource {
            Ok(resource) => Ok(resource),
            Err(e) => invalid_data_error(&format!("{}", e)),
        }
    }
}
