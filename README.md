# Jlc Discord Bot
Simple discord bot to check JLC component stock

# Usage
* Install cargo
* Fill out `.env` file
  * Put discord token in example file and rename it
* Run the bot with `cargo run` and install dependencies if it requires any

# Running as Daemon
This will be for users with systemd, if you are using another init system, you will need to modify these files
1. Fill out the `start.sh` and `jlc.service` files with the necessary information, move to main directory
2. Run `chmod +x start.sh`
3. Edit `components.json` to contain the correct components and role/channel IDs and move to config folder
4. Move `jlc.service` to `/lib/systemd/service/`
5. Allow `start.sh` to be executed by running `chmod +x start.sh`
6. Enable the service with `sudo systemctl enable jlc`
   1. If you're doing this for the first time, run `sudo systemctl start jlc` to start it
   
## Useful commands and utilities 
As and IDE I highly recommend IntelliJ and/or CLion, with the Rust plugin of course

To test JLC's API I've been using [Postman](https://www.postman.com/)