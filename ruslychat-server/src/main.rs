use mysql::prelude::*;
use mysql::*;

fn main() -> Result<()> {

    let url: &str = "mysql://test:test@localhost:3306/test";
    let opts: Opts = Opts::from_url(url)?;

    let pool: Pool = Pool::new(opts)?;
    let mut conn: PooledConn = pool.get_conn()?;

    Ok(())
}