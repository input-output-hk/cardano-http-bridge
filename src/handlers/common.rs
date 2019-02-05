use super::super::config::{Network, Networks};
use cardano::block::EpochId;
use iron::Request;
use router::Router;

pub fn validate_network_name(v: &&str) -> bool {
    v.chars().all(|c| c.is_ascii_alphanumeric())
}

pub fn validate_epochid(v: &&str) -> Option<EpochId> {
    if !v.chars().all(|c| c.is_digit(10)) {
        None
    } else {
        Some(v.parse::<EpochId>().unwrap())
    }
}

pub fn get_network<'a>(req: &Request, networks: &'a Networks) -> Option<(String, &'a Network)> {
    let ref network_name = req
        .extensions
        .get::<Router>()
        .unwrap()
        .find("network")
        .unwrap();

    if !validate_network_name(network_name) {
        return None;
    }

    match networks.get(network_name.to_owned()) {
        None => None,
        Some(net) => Some((network_name.to_string(), net)),
    }
}

pub fn get_network_and_epoch<'a>(
    req: &Request,
    networks: &'a Networks,
) -> Option<(&'a Network, EpochId)> {
    let (_, net) = get_network(req, networks)?;
    let ref epochid_str = req
        .extensions
        .get::<Router>()
        .unwrap()
        .find("epochid")
        .unwrap();

    let epochid = match validate_epochid(epochid_str) {
        None => {
            error!("invalid epochid: {}", epochid_str);
            return None;
        }
        Some(e) => e,
    };

    Some((net, epochid))
}
