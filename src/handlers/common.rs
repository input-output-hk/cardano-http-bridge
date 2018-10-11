use cardano::block::EpochId;
use router::{Router};
use iron::{Request};
use super::super::config::{Networks, Network};

pub fn validate_network_name(v: &&str) -> bool {
    v.chars().all(|c| c.is_ascii_alphanumeric())
}

pub fn validate_epochid(v: &&str) -> Option<EpochId> {
    if ! v.chars().all(|c| c.is_digit(10)) {
        None
    } else {
        Some(v.parse::<EpochId>().unwrap())
    }
}

pub fn get_network_and_epoch<'a>(req: &Request, networks: &'a Networks) -> Option<(&'a Network, EpochId)> {
    let ref network_name = req.extensions.get::<Router>().unwrap().find("network").unwrap();
    let ref epochid_str = req.extensions.get::<Router>().unwrap().find("epochid").unwrap();

    if ! validate_network_name (network_name) {
        return None;
    }
    let net = match networks.get(network_name.to_owned()) {
        None => return None,
        Some(net) => net
    };

    let epochid = match validate_epochid (epochid_str) {
        None => {
            error!("invalid epochid: {}", epochid_str);
            return None;
        },
        Some(e) => e,
    };

    Some((net, epochid))
}
