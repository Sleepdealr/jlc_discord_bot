# Jlc Discord Bot
Simple discord bot to check JLC component stock

#  Usage
* Install cargo
* Fill out `.env` file
  * Put discord token in example file and rename it
* Run the bot with `cargo run` and install dependencies if it requires any

# Running as Daemon
This will be for users with systemd, if you are using another init system, you will need to modify these files
1. Fill out the `start.sh` and `jlc.service` files with the necessary information
2. Move `jlc.service` to `/lib/systemd/service/`
3. Allow `start.sh` to be executed by running `chmod +x start.sh`
4. Enable the service with `sudo systemctl enable jlc`
   1. If you're doing this for the first time, run `sudo systemctl start jlc` to start it
   