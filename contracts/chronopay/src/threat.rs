use soroban_sdk::{contracttype, Env, Vec};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Threat {
    Reentrancy,
    Overflow,
    UnauthorizedAccess,
    InconsistentState,
    DenialOfService,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ThreatChecklist {
    pub mitigations: Vec<Threat>,
}

pub fn add_mitigation(env: &Env, checklist: &mut ThreatChecklist, threat: Threat) {
    if !checklist.mitigations.iter().any(|x| x == threat) {
        checklist.mitigations.push_back(threat);
    }
}
