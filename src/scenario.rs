//! Scenario Config
//!
//! This config describes the relayer game scenario
//!
//! Some examples are listed in `/scenario `
//!
//! surfix d: block difference between last block number relayed on Darwinia,
//! surfix e: block difference between last related block number of Ethereum
use std::iter::IntoIterator;
use std::str::FromStr;

use serde_derive::Deserialize;
use toml;

use crate::bond::{
    linear::LinearConfig as BondLinear, ConfigValidate as BondVali, Equation as BondEq,
};
use crate::challenge::{
    linear::LinearConfig as ChallengeLinear, ConfigValidate as ChallengeVali,
    Equation as ChallengeEq,
};
use crate::error::Error;
use crate::target::{half::HalfConfig, Equation as TargetEq};

/// # Scenario Config
/// In this config, the `challenge_function`, the initial status, and the `relayers` are defined.
/// The initaial status contains the block difference in the target chain and Darwinia chain.
#[allow(non_snake_case)]
#[derive(Debug, Deserialize)]
pub struct ScenarioConfig {
    pub title: Option<String>,
    /// Dd: (optional) the initial block difference between last block number relayed on Darwinia, default 0
    pub Dd: Option<usize>,
    /// De: the initial block difference between last related block number of target chain (for example Ethereum), default 100
    pub De: Option<usize>,

    /// F: The block producing factor for darwinia / ethereum
    /// For example, 2.0 means that darwinia produce 2 blocks and ethereum produce 1 block.
    pub F: Option<f64>,

    /// Once a relayer submit a header and wait the challenge time in blocks after the calculated value equation from challenge
    /// function, Darwinia network will deem this header is valided and become a last relayed header.
    pub challenge_function: String,

    /// Once there is disput on any header, the relayer should submit the next ethere block target as
    /// calculated.
    pub target_function: String,

    /// The submit bond of function
    pub bond_function: String,

    /// parameters in linear wating
    pub challenge_linear: Option<ChallengeLinear>,

    /// parameters in linear wating
    pub bond_linear: Option<BondLinear>,

    /// The relayers participate in these game
    /// We suppose that there is always a honest relayer provied by Darwinia,
    /// so after the config correctly imported, the Darwinia relayer will add into.
    pub relayers: Vec<RelayerConfig>,
}

pub struct ScenarioConfigIntoIterator {
    relayers: Vec<RelayerConfig>,
    pub submit_round: usize,
}

impl IntoIterator for ScenarioConfig {
    type Item = Vec<(String, bool)>;
    type IntoIter = ScenarioConfigIntoIterator;

    fn into_iter(self) -> Self::IntoIter {
        ScenarioConfigIntoIterator {
            relayers: self.relayers,
            submit_round: 0,
        }
    }
}

impl ScenarioConfig {
    /// a method to get iterator with less clone, and save memory
    pub fn get_iter(&self) -> ScenarioConfigIntoIterator {
        ScenarioConfigIntoIterator {
            relayers: self.relayers.clone(),
            submit_round: 0,
        }
    }
    pub fn get_challenge_equation(&self) -> Result<Box<dyn ChallengeEq>, Error> {
        if let Ok(i) = self.challenge_function.as_str().parse::<usize>() {
            return Ok(Box::new(i));
        }
        match self.challenge_function.to_uppercase().as_str() {
            "LINEAR" => {
                if let Some(w) = self.challenge_linear {
                    return Ok(Box::new(w));
                }
            }
            _ => {
                return Err(Error::ParameterError("Challenge function not support"));
            }
        }
        return Err(Error::ParameterError(
            "lack prameters for specified challenge function",
        ));
    }
    pub fn get_target_equation(&self) -> Result<impl TargetEq, Error> {
        match self.target_function.to_uppercase().as_str() {
            "HALF" => return Ok(HalfConfig {}),
            _ => {
                return Err(Error::ParameterError("Target function absent"));
            }
        }
    }
    pub fn get_bond_equation(&self) -> Result<Box<dyn BondEq>, Error> {
        if let Ok(i) = self.bond_function.as_str().parse::<f64>() {
            return Ok(Box::new(i));
        }
        match self.bond_function.to_uppercase().as_str() {
            "LINEAR" => {
                if let Some(f) = self.bond_linear {
                    return Ok(Box::new(f));
                }
            }
            _ => {
                return Err(Error::ParameterError("Bond function not support"));
            }
        }
        return Err(Error::ParameterError(
            "lack prameters for specified bond function",
        ));
    }
    pub fn apply_patch(&mut self, patches: Vec<&str>) -> Result<(), Error> {
        let split_with_equation = patches.iter().map(|patch| {
            let mut i = patch.split('=').into_iter();
            return (i.clone().nth(0), i.nth(1));
        });
        for key_values in split_with_equation {
            if let (Some(k), Some(v)) = key_values {
                let para = k.split('.').nth(1);
                if k.starts_with("challenge_linear") {
                    let p = para.ok_or_else(|| {
                        Error::PatchParameterError("challenge linear parameter absent".to_string())
                    })?;
                    let mut w = self.challenge_linear.ok_or_else(|| {
                        Error::PatchParameterError("challenge linear absent".to_string())
                    })?;
                    w.apply_patch(p, v)?;
                    w.validate()?;
                    self.challenge_linear = Some(w);
                } else if k.starts_with("bond_linear") {
                    let p = para.ok_or_else(|| {
                        Error::PatchParameterError("bond linear parameter absent".to_string())
                    })?;
                    let mut f = self.bond_linear.ok_or_else(|| {
                        Error::PatchParameterError("bond linear absent".to_string())
                    })?;
                    f.apply_patch(p, v)?;
                    f.validate()?;
                    self.bond_linear = Some(f);
                } else if k.starts_with("challenge_function") {
                    self.challenge_function = v.to_string();
                } else if k.starts_with("bond_function") {
                    self.bond_function = v.to_string();
                }
            } else {
                return Err(Error::PatchParameterError(
                    key_values.0.unwrap_or_default().to_string(),
                ));
            }
        }
        Ok(())
    }
}

impl Iterator for ScenarioConfigIntoIterator {
    type Item = Vec<(String, bool)>;
    /// Return the the relayer is liing in each round
    fn next(&mut self) -> Option<Vec<(String, bool)>> {
        if self.submit_round >= self.relayers[0].choice.len() {
            return None;
        }
        let current_index = self.submit_round;
        self.submit_round += 1;
        Some(
            self.relayers
                .iter()
                .map(|r| {
                    if current_index >= r.choice.len() {
                        Err("this replay has no response")
                    } else {
                        Ok((
                            r.name.clone().unwrap(),
                            r.choice.chars().nth(current_index).unwrap() == 'L',
                        ))
                    }
                })
                .filter_map(Result::ok)
                .collect(),
        )
    }
}

/// RelayerConfig
/// Set up a `name` and the `choice` about the relayer
#[derive(Debug, Deserialize, Clone)]
pub struct RelayerConfig {
    /// Optional field help you to know the relayer in
    pub name: Option<String>,
    /// The client can choice to be Honest(H), Lie(L), No response(N), if the choice is not lone as
    /// other replayer, it will be automaticaly no response
    pub choice: String,
}

impl FromStr for ScenarioConfig {
    type Err = Error;
    /// Pase from the scenario toml file as listed in `/scenario `
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut c: ScenarioConfig = toml::from_str(s)?;
        c.challenge_function.make_ascii_uppercase();
        if let Some(w) = c.challenge_linear {
            w.validate()?;
        }
        if let Some(f) = c.bond_linear {
            f.validate()?;
        }

        let mut max_chose = 0;
        for (i, r) in c.relayers.iter_mut().enumerate() {
            if let Some(n) = &r.name {
                if n.to_uppercase() == "DARWINIA".to_string() {
                    return Err(Error::ParameterError(
                        "Darwinia relayer alway honest, you do not need modify it",
                    ));
                }
            } else {
                r.name = Some(format!(" {}", i));
            };
            // TODO: check name should be use number
            r.choice.make_ascii_uppercase();
            max_chose = std::cmp::max(max_chose, r.choice.len());
            for c in r.choice.chars() {
                if c != 'H' && c != 'L' && c != 'N' {
                    return Err(Error::ParameterError("relayer chose must be 'H', 'L', 'N'"));
                }
            }
        }
        let mut relayers = vec![RelayerConfig {
            name: Some("Darwinia".to_string()),
            choice: "H".repeat(max_chose + 1),
        }];
        relayers.append(&mut c.relayers);
        c.relayers = relayers;
        Ok(c)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    static TOML_CONFIG: &'static str = r#"
			challenge_function = "linear"
			target_function = "half"
			bond_function = "10.0"

			Dd = 100
			De = 1000

			[challenge_linear]
			Wd = 0.0
			We = 0.0
			C  = 1
			Md = 100
			Me = 100
			B = 1
			T = 10

			[[relayers]]
			name = "Evil"
			choice = "LL"

			[[relayers]]
			name = "Honest"
			choice = "H"
		"#;
    #[test]
    fn test_parse_toml() {
        let c: ScenarioConfig = toml::from_str(TOML_CONFIG).unwrap();
        assert_eq!(c.relayers.len(), 2);
        assert_eq!(c.relayers[1].choice, "H");
    }
    #[test]
    fn test_from_toml_str() {
        let c = <ScenarioConfig>::from_str(TOML_CONFIG);
        assert!(c.is_ok());
        let c = c.unwrap();
        assert_eq!(c.relayers.len(), 3);

        // Darwinia relayer should be added automatically
        assert_eq!(c.relayers[0].name, Some("Darwinia".to_string()));
        assert_eq!(c.relayers[0].choice, "HHH");
        assert_eq!(c.relayers[1].choice, "LL");
        assert_eq!(c.relayers[2].choice, "H");

        let bond_function = c.get_bond_equation();
        assert!(bond_function.is_ok());
        assert_eq!(bond_function.unwrap().calculate(0), 10.0);
    }
    #[test]
    fn test_iterate_from_scenario() {
        let c = <ScenarioConfig>::from_str(TOML_CONFIG).unwrap();
        let mut i = c.into_iter();
        assert_eq!(
            i.next(),
            Some(vec![
                ("Darwinia".to_string(), false),
                ("Evil".to_string(), true),
                ("Honest".to_string(), false)
            ])
        );
        assert_eq!(
            i.next(),
            Some(vec![
                ("Darwinia".to_string(), false),
                ("Evil".to_string(), true),
            ])
        );
        assert_eq!(i.next(), Some(vec![("Darwinia".to_string(), false),]));
        assert_eq!(i.next(), None);
    }
    #[test]
    fn test_from_error_toml_str() {
        let c = <ScenarioConfig>::from_str(
            r#"
			challenge_function = "linear"
			target_function = "half"
			bond_function = "10.0"

			[challenge_linear]
			Wd = -10
			We = 0.0
			C  = 1
			Md = 100
			Me = 100
			B = 1
			T = 10

			[[relayers]]
			name = "Honest"
			choice = "HHHHHHH"
		"#,
        );
        assert!(format!("{:?}", c).contains("Wd should not be negative"));
    }
    #[test]
    fn test_auto_upper_case() {
        let c = <ScenarioConfig>::from_str(
            r#"
			challenge_function = "linear"
			target_function = "half"
			bond_function = "10.0"

			[challenge_linear]
			Wd = 0.0
			We = 0.0
			C  = 1
			Md = 100
			Me = 100
			B = 1
			T = 10

			[[relayers]]
			name = "Honest"
			choice = "hhhhhhh"
			"#,
        )
        .unwrap();
        assert_eq!(c.relayers[1].choice, "HHHHHHH");
    }
    #[test]
    fn test_apply_patch() {
        let mut c = <ScenarioConfig>::from_str(TOML_CONFIG).unwrap();
        c.apply_patch(vec!["challenge_function=9487"]).unwrap();
        let challenge_function = c.get_challenge_equation();
        assert!(challenge_function.is_ok());
        assert_eq!(challenge_function.unwrap().calculate(10, 10), 9487);

        c.apply_patch(vec!["bond_function=1.2222"]).unwrap();
        let bond_function = c.get_bond_equation();
        assert!(bond_function.is_ok());
        assert_eq!(bond_function.unwrap().calculate(0), 1.2222);
    }
}
