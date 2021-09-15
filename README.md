A simple bot to demonstrate `ruma-client` functionality. Tells jokes when you ask for them.
# Usage

Create a file called `config` and populate it with the following values:
- `homeserver`: Your homeserver URL
- username: The Matrix ID for the bot
- password: 

For example:

```ini
homeserver=http://example.com:8080/
username=@user:example.com
password=yourpassword
```

You will need to pre-register the bot account; it doesn't do registration
automatically. The bot will automatically join rooms it is invited to though.

Finally, run the bot (e.g. using `cargo run`) from the same directory as your
`config` file. The bot should respond to the request "Tell me a joke" in any
channel that it is invited to.