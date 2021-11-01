use crate::messages::*;
use actix::prelude::*;

pub struct Printer;

impl Actor for Printer {
    type Context = Context<Self>;
}

impl Handler<StockInfo> for Printer {
    type Result = ();

    fn handle(&mut self, info: StockInfo, _: &mut Context<Self>) {
        println!("{}", info.fmt_csv());
    }
}
