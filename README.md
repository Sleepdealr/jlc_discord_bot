# Jlc Discord Bot
Simple discord bot to check JLC component stock

# Usage
* Install cargo
* Fill out `.env` file
  * Put discord token in example file and rename it
* Run the bot with `cargo run` and install dependencies if it requires any

# Running as Daemon
This will be for users with systemd, if you are using another init system, you will need to modify these files
1. Fill out the `start.sh` and `jlc.service` files with the necessary information
2. Edit `components.json` to contain the correct components and role/channel IDs
3. Move `jlc.service` to `/lib/systemd/service/`
4. Allow `start.sh` to be executed by running `chmod +x start.sh`
5. Enable the service with `sudo systemctl enable jlc`
   1. If you're doing this for the first time, run `sudo systemctl start jlc` to start it
   
## Useful commands and utilities 
As and IDE I highly recommend IntelliJ and/or CLion, with the Rust plugin of course

To test JLC's API I've been using [Postman](https://www.postman.com/) which is an amazing utility