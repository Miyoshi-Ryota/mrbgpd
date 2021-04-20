use rtnetlink::{new_connection, Error, Handle, IpVersion};
use rtnetlink::packet::rtnl::RouteMessage;
use futures::stream::{self, TryStreamExt};
use std::str::FromStr;
use std::net::{Ipv4Addr, IpAddr};
use std::net::AddrParseError;

#[derive(Debug, Clone, Copy)]
pub struct IpPrefix {
    network_address: Ipv4Addr, // ToDo: 正確にはネットワークアドレス的なやつなのでipv4addrを使うのは不適切
    prefix_length: u8,
}

impl IpPrefix {
    pub fn new(network_address: Ipv4Addr, prefix_length: u8) -> Self {
        Self {
            network_address,
            prefix_length,
        }
    }

    pub fn decode(&self) -> Vec<u8> {
        let network_address = self.network_address.octets();
        let mut result = vec![self.prefix_length];
        if self.prefix_length == 0 {
            return result;
        }
        if self.prefix_length > 0 {
            result.push(network_address[0]);
        };
        if self.prefix_length > 8 {
            result.push(network_address[1]);
        };
        if self.prefix_length > 16 {
            result.push(network_address[2]);
        }
        if self.prefix_length > 24 {
            result.push(network_address[3]);
        }
        result
    }

    pub fn does_include(&self, other: &Self) -> bool {
        // 192.168.0.0 / 16
        // same.same.0.0
        // 192.168.5.0 / 24
        // same.same.00000101.0
        if self.prefix_length > other.prefix_length {
            return false;
        };
        let mut self_bits: String = String::from("");
        let mut other_bits: String = String::from("");

        let other_octate = other.network_address.octets();
        for i in self.network_address.octets().iter() {
            self_bits.push_str(&format!("{:8b}", i));
        }
        for i in other.network_address.octets().iter() {
            other_bits.push_str(&format!("{:8b}", i));
        }

        for i in 0..self.prefix_length {
            if self_bits.chars().nth(i as usize) != other_bits.chars().nth(i as usize) {
                return false;
            }
        }
        true
    }
}

impl FromStr for IpPrefix {
    type Err = AddrParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let split = s.split('/').collect::<Vec<_>>();
        let network_address: Ipv4Addr = split[0].parse().unwrap();
        let prefix_length: u8 = split[1].parse().unwrap();
        Ok(Self {network_address, prefix_length,})
    }
}

async fn routing_table_example() -> Result<(), ()> {
    let (connection, handle, _) = new_connection().unwrap();
    tokio::spawn(connection);

    println!("dumping routes for IPv4");
    if let Err(e) = dump_addresses(handle.clone(), IpVersion::V4).await {
        eprintln!("{}", e);
    }
    println!();

    println!("dumping routes for IPv6");
    if let Err(e) = dump_addresses(handle.clone(), IpVersion::V6).await {
        eprintln!("{}", e);
    }
    println!();

    Ok(())
}

fn print_typename<T>(_: &T) {
    println!("{}", std::any::type_name::<T>());
}

pub async fn get_all_ip_v4_routes() -> Result<Vec<RouteMessage>, Error> {
    let (connection, handle, _) = new_connection().unwrap();
    tokio::spawn(connection);
    let mut routes = handle.route().get(IpVersion::V4).execute();
    let mut result = vec![];
    while let Some(route) = routes.try_next().await? {
        result.push(route);
    }
    Ok(result)
}

pub async fn lookup_network_route(ip_prefix: &IpPrefix) -> Result<Vec<RouteMessage>, Error> {
    let all_routes = get_all_ip_v4_routes().await.unwrap();
    let mut result = vec![];
    for route in all_routes {
        let (network_address, prefix_length) = route.destination_prefix().unwrap();
        let network_address = match network_address {
            IpAddr::V4(addr) => addr,
            IpAddr::V6(_) => panic!(),
        };
        let prefix = IpPrefix {
            network_address, prefix_length,
        };
        if ip_prefix.does_include(&prefix) {
            result.push(route);
        }
    }
    Ok(result)
}

async fn dump_addresses(handle: Handle, ip_version: IpVersion) -> Result<(), Error> {
    let mut routes = handle.route().get(ip_version).execute();
    while let Some(route) = routes.try_next().await? {
        print_typename(&route);
        println!("{:?}", route);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn it_works() {
        routing_table_example().await;
        assert_eq!(2 + 2, 4);
    }

    #[tokio::test]
    async fn test_does_ip_prefix_include() {
        let bigger_ip_prefix = IpPrefix {
            network_address: Ipv4Addr::new(192, 168,  0,  0),
            prefix_length: 16,
        };

        let smaller_ip_prefix = IpPrefix {
            network_address: Ipv4Addr::new(192, 168, 5, 0),
            prefix_length: 24,
        };

        assert_eq!(bigger_ip_prefix.does_include(&smaller_ip_prefix), true);
        assert_eq!(smaller_ip_prefix.does_include(&bigger_ip_prefix), false);
    }
}
