use yogurt::argument::parser::IntArgument;
use yogurt::{Command, Dispatcher};

fn main() {
    // Create a dispatcher
    let dispatcher =
        Dispatcher::builder()
            // command prefix, defaults to none
            .prefix("/")
            .base_context(())
            // context factory, new context is created for every executed command
            .context_factory(|_| ())
            .child(Command::literal("ping").child(
                Command::argument("number", IntArgument, true).exec(|ctx| {
                    println!("{:?}", ctx);
                    Ok(())
                }),
            ))
            .build()
            // fails if no context factory provided
            .unwrap();

    // run command
    dispatcher.run_command("/ping 3").unwrap();
}
