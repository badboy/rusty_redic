#[macro_use]
extern crate ruru;
#[macro_use]
extern crate lazy_static;

extern crate redis;

use std::str;
use std::error::Error;
use ruru::{Class, Object, RString, Array, AnyObject, NilClass, VM, Fixnum};

pub struct Redic {
    client: redis::Client,
    queue: Vec<Vec<String>>,
}

impl Redic {
    pub fn new(client: redis::Client) -> Redic {
        Redic {
            client: client,
            queue: vec![]
        }
    }

    pub fn client(&self) -> &redis::Client {
        &self.client
    }

    pub fn queue(&mut self) -> &mut Vec<Vec<String>> {
        &mut self.queue
    }
}

wrappable_struct!(Redic, RedicWrapper, REDIC_WRAPPER);

class!(RustyRedic);

fn redis_value_to_any_object(val: redis::Value) -> AnyObject {
    match val {
        redis::Value::Nil => NilClass::new().to_any_object(),
        redis::Value::Int(i) => Fixnum::new(i).to_any_object(),
        redis::Value::Data(vec) => {
            unsafe {
                let s = str::from_utf8_unchecked(&vec);
                RString::new(s).to_any_object()
            }
        }
        redis::Value::Bulk(vec) => {
            let mut v = Array::new();
            for elem in vec {
                v.push(redis_value_to_any_object(elem));
            }
            v.to_any_object()
        }
        redis::Value::Status(s) => RString::new(&s).to_any_object(),
        redis::Value::Okay => RString::new("OK").to_any_object()
    }
}

methods!(
    RustyRedic,
    itself,

    fn redic_new(url: RString) -> AnyObject {
        let default_url = "redis://localhost:6379";
        let url = url
            .map(|url| url.to_string())
            .unwrap_or_else(|_| default_url.to_string());
        let client = match redis::Client::open(&*url) {
            Ok(client) => client,
            Err(_)     => {
                VM::raise(Class::from_existing("ArgumentError"), "Can't create RustyRedic");
                unreachable!();
            }
        };
        let redic = Redic::new(client);

        Class::from_existing("RustyRedic")
            .wrap_data(redic, &*REDIC_WRAPPER)
    }

    fn redic_call(args: Array) -> AnyObject {
        let args = match args {
            Err(error) => {
                VM::raise(error.to_exception(), error.description());
                unreachable!();
            }
            Ok(args) => args
        };
        if args.length() == 0 {
            VM::raise(Class::from_existing("ArgumentError"), "Need atleast 1 argument");
            unreachable!();
        }

        let client = itself.get_data(&*REDIC_WRAPPER).client();
        let con = match client.get_connection() {
            Ok(con) => con,
            Err(_)     => {
                VM::raise(Class::from_existing("ArgumentError"), "Can't create connection");
                unreachable!();
            }
        };

        let mut args = args.into_iter()
            .map(|obj| {
                let obj = match obj.try_convert_to::<RString>() {
                    Ok(obj) => obj,
                    Err(_)     => {
                        VM::raise(Class::from_existing("ArgumentError"), "Can't coerce to String");
                        unreachable!();
                    }
                };
                obj.to_string()
            });

        let cmd = args.next().unwrap();
        let mut cmd = redis::cmd(&cmd);
        for arg in args {
            cmd.arg(&arg);
        }

        let res : redis::RedisResult<redis::Value> = cmd.query(&con);

        match res {
            Err(_) => {
                VM::raise(Class::from_existing("ArgumentError"), "Can't execute command");
                unreachable!();
            }
            Ok(val) => redis_value_to_any_object(val)
        }
    }

    fn redic_queue(args: Array) -> NilClass {
        let args = match args {
            Err(error) => {
                VM::raise(error.to_exception(), error.description());
                unreachable!();
            }
            Ok(args) => args
        };
        if args.length() == 0 {
            VM::raise(Class::from_existing("ArgumentError"), "Need atleast 1 argument");
            unreachable!();
        }

        let args = args.into_iter()
            .map(|obj| {
                let obj = match obj.try_convert_to::<RString>() {
                    Ok(obj) => obj,
                    Err(_)     => {
                        VM::raise(Class::from_existing("ArgumentError"), "Can't coerce to String");
                        unreachable!();
                    }
                };
                obj.to_string()
            }).collect::<Vec<String>>();

        let mut queue = itself.get_data(&*REDIC_WRAPPER).queue();
        queue.push(args);

        NilClass::new()

    }

    fn redic_commit() -> AnyObject {
        let mut queue = itself.get_data(&*REDIC_WRAPPER).queue();
        if queue.is_empty() {
            return NilClass::new().to_any_object();
        }

        let client = itself.get_data(&*REDIC_WRAPPER).client();
        let con = match client.get_connection() {
            Ok(con) => con,
            Err(_)     => {
                VM::raise(Class::from_existing("ArgumentError"), "Can't create connection");
                unreachable!();
            }
        };

        let mut pipe = redis::pipe();
        for cmd in queue.drain(..) {
            let mut args = cmd.iter();
            let first = args.next().unwrap();
            pipe.cmd(first);
            for arg in args {
                pipe.arg(arg);
            }
        }

        let res : redis::RedisResult<redis::Value> = pipe.query(&con);

        match res {
            Err(_) => {
                VM::raise(Class::from_existing("ArgumentError"), "Can't execute command");
                unreachable!();
            }
            Ok(val) => redis_value_to_any_object(val)
        }
    }
);

#[no_mangle]
pub extern fn init_rusty_redic() {
    let data_class = Class::from_existing("Data");

    Class::new("RustyRedic", Some(&data_class)).define(|itself| {
        itself.def_self("new", redic_new);

        itself.def("call", redic_call);
        itself.def("queue", redic_queue);
        itself.def("commit", redic_commit);
    });
}
