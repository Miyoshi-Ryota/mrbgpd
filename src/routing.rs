use rtnetlink::{new_connection, Error, Handle, IpVersion};
use rtnetlink::packet::rtnl::RouteMessage;
use futures::stream::{self, TryStreamExt};


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
}
