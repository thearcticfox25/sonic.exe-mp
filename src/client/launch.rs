//! The game's command line: where the game starts.

use crate::client::levels::LEVELS;
use crate::client::net::random_alone_pick;

/// The names that ask for the game. It starts without them too; they only say so.
pub const CLIENT_FLAGS: [&str; 3] = ["--play", "--client", "--game"];

pub const USAGE: &str = "\
AdvancedServer [--play] [options]  the game, also with no options at all
                                   (--client and --game say --play too)
  --map N                     straight into level N alone (1..21, the server's map
                              numbers); leave in the pause menu goes to the main menu
  --character NAME            with --map: tails, knuckles, eggman, amy, cream, sally, exe,
                              chaos, exetior, exeller, random (default) or freecam
  --join [HOST[:PORT]]        straight into that server's lobby, the address saved as
                              the join page's; without one, the saved address
  --lobby N                   with --join: lobby N of the server (1 is the first), as
                              the .lobby command picks it: the port after the first
  --netlog                    online: print the camera and the own player every 60
                              ticks, and every correction the server makes to it
  --observe HOST:PORT --observer-key KEY
                              the server's ?gui window, which the server starts itself:
                              watching its lobby, without sound
AdvancedServer --host              the server (--server and --dedicated too)
AdvancedServer --update            download the latest release, asking first
AdvancedServer --help              this text";

pub enum Start {
    /// The logos, then the main menu, as always.
    Logo,
    /// A level alone: its number in LEVELS, the pick for NetClient::choose_alone and
    /// whether it is the free camera.
    Alone { level: usize, pick: (u8, bool, bool), freecam: bool },
    /// A server's lobby: the address (None means the saved one) and the lobby number.
    Join(Option<String>, Option<u16>),
    /// The server's ?gui window (states::host_gui): its address and the key it was given.
    Observe { address: String, key: String },
}

pub struct Launch {
    pub start: Start,
    pub netlog: bool,
}

impl Launch {
    /// Reads the arguments (without the program's name). The message says what is wrong.
    pub fn parse(arguments: &[String]) -> Result<Launch, String> {
        let mut netlog = false;
        let mut map = None;
        let mut character = None;
        let mut join = None;
        let mut lobby = None;
        let mut observe = None;
        let mut observer_key = None;
        let mut index = 0;
        while index < arguments.len() {
            let argument = arguments[index].as_str();
            match argument {
                // Only saying what the program is started as: the game is what it runs anyway.
                flag if CLIENT_FLAGS.contains(&flag) => {}
                "--map" => map = Some(take_value(arguments, &mut index, argument)?),
                "--character" => character = Some(take_value(arguments, &mut index, argument)?),
                "--join" => join = Some(take_value(arguments, &mut index, argument).ok()),
                "--lobby" => lobby = Some(take_value(arguments, &mut index, argument)?),
                "--observe" => observe = Some(take_value(arguments, &mut index, argument)?),
                "--observer-key" => observer_key = Some(take_value(arguments, &mut index, argument)?),
                "--netlog" => netlog = true,
                _ => return Err(format!("unknown option {argument}")),
            }
            index += 1;
        }

        if let Some(address) = observe {
            let key = observer_key.ok_or("--observe needs --observer-key")?;
            return Ok(Launch { start: Start::Observe { address, key }, netlog });
        }
        if lobby.is_some() && join.is_none() {
            return Err("--lobby needs --join".to_string());
        }
        let lobby = match lobby {
            Some(text) => match text.parse::<u16>() {
                Ok(number) if number >= 1 => Some(number),
                _ => return Err(format!("--lobby: {text} is not a lobby number (1, 2, ...)")),
            },
            None => None,
        };
        let start = match (map, character, join) {
            (Some(_), _, Some(_)) => return Err("--map and --join cannot be used together".to_string()),
            (None, Some(_), _) => return Err("--character needs --map".to_string()),
            (Some(map), character, None) => {
                let level: usize = map.parse().map_err(|_| format!("--map: {map} is not a number"))?;
                if level >= LEVELS.len() {
                    return Err(format!("--map is 1..{}", crate::maps::LAST_MAP));
                }
                let (pick, freecam) = character_pick(character.as_deref().unwrap_or("random"))?;
                Start::Alone { level, pick, freecam }
            }
            (None, None, Some(address)) => Start::Join(address, lobby),
            (None, None, None) => Start::Logo,
        };
        Ok(Launch { start, netlog })
    }
}

/// The value after the option at `index`, stepping over it. A value never starts with
/// "--", so `--join --netlog` is a --join without an address.
fn take_value(arguments: &[String], index: &mut usize, option: &str) -> Result<String, String> {
    match arguments.get(*index + 1).filter(|value| !value.starts_with("--")) {
        Some(value) => {
            *index += 1;
            Ok(value.clone())
        }
        None => Err(format!("{option} needs a value")),
    }
}

/// (character, is_exe, demonized) and whether it is the free camera, as the
/// Sandbox page's buttons give them (packet::NO_CHARACTER and on).
fn character_pick(name: &str) -> Result<((u8, bool, bool), bool), String> {
    let pick = match name {
        "tails" => (1, false, false),
        "knuckles" => (2, false, false),
        "eggman" => (3, false, false),
        "amy" => (4, false, false),
        "cream" => (5, false, false),
        "sally" => (6, false, false),
        "exe" => (1, true, false),
        "chaos" => (2, true, false),
        "exetior" => (3, true, false),
        "exeller" => (4, true, false),
        "random" => random_alone_pick(),
        // The free camera plays nobody; the level builds its room around Tails.
        "freecam" => return Ok(((crate::packet::NO_CHARACTER, false, false), true)),
        _ => return Err(format!("unknown character {name}")),
    };
    Ok((pick, false))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(line: &str) -> Result<Launch, String> {
        Launch::parse(&line.split_whitespace().map(String::from).collect::<Vec<_>>())
    }

    #[test]
    fn the_command_line_says_where_the_game_starts() {
        assert!(matches!(parse("--play").unwrap().start, Start::Logo));
        assert!(matches!(parse("--play --map 3 --character chaos").unwrap().start, Start::Alone { level: 3, pick: (2, true, false), freecam: false }));
        assert!(matches!(parse("--play --map 0 --character freecam").unwrap().start, Start::Alone { freecam: true, .. }));
        let observe = parse("--play --observe 127.0.0.1:7606 --observer-key abc").unwrap().start;
        assert!(matches!(observe, Start::Observe { address, key } if address == "127.0.0.1:7606" && key == "abc"));
        assert!(parse("--observe 127.0.0.1:7606").is_err(), "the window needs its key");
        assert!(matches!(parse("--play --join example.org:7607").unwrap().start, Start::Join(Some(ref address), None) if address == "example.org:7607"));
        assert!(matches!(parse("--play --join --netlog").unwrap().start, Start::Join(None, None)));
        assert!(matches!(parse("--play --join --lobby 2").unwrap().start, Start::Join(None, Some(2))));
        assert!(parse("--play --lobby 2").is_err());
        assert!(parse("--play --join --lobby 0").is_err());
        assert!(parse("--play --character amy").is_err());
        assert!(parse("--play --map 22").is_err());
        assert!(matches!(parse("--play --map 0").unwrap().start, Start::Alone { level: 0, .. }), "the cut level, unsaid");
        assert!(parse("--play --map 1 --join").is_err());
        assert!(parse("--play --shot 5").is_err());
        assert!(matches!(parse("--client").unwrap().start, Start::Logo));
        assert!(matches!(parse("--game --map 3").unwrap().start, Start::Alone { level: 3, .. }));
    }
}
