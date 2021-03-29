use rtnetlink::{new_connection, Error, Handle, IpVersion};
use futures::stream::{self, TryStreamExt};

async fn aaa() -> Result<(), ()> {
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
        aaa().await;
        assert_eq!(2 + 2, 4);
    }
}
