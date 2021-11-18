use std::{
    collections::HashMap,
    io::{
        BufRead,
        BufReader,
        Read,
        Seek
    },
};

// Assumed to be 4k osu!mania
#[derive(Debug)]
pub struct Map {
    pub audio_file_name: String,
    pub audio_lead_in: usize,
    pub full_title: String,
    pub notes: [Vec<usize>; 4],
}

#[derive(Debug)]
pub struct Set {
    pub maps: Vec<Map>,
    pub files: HashMap<String, Vec<u8>>
}

impl Set {
    pub fn from_osz<R>(reader: R) -> Set where
        R: Read+Seek
    {
        let mut archive = zip::ZipArchive::new(reader).unwrap();
        let mut maps = Vec::new();
        let mut files = HashMap::new();
        for i in 0..archive.len() {
            let mut file = archive.by_index(i).unwrap();
            let extention = std::path::Path::new(file.name()).extension();
            match extention {
                Some(extention) if extention.to_ascii_lowercase().to_str().unwrap() == "osu" => {
                    if true {
                            eprintln!("Parsing {:#?} as a osu map", file.name());

                            struct HitCircle {
                                x: usize,
                                time: usize
                            }

                            let mut valid = true;
                            let mut audio_file_name: Option<String> = None;
                            let mut audio_lead_in: usize = 0;
                            let mut mode: Option<usize> = None;
                            let mut title: Option<String> = None;
                            let mut version: Option<String> = None;
                            let mut column_count: Option<usize> = None;
                            let mut hit_circles: Vec<HitCircle> = Vec::new();
                            enum ParseState {
                                None,
                                General,
                                Editor,
                                Metadata,
                                Difficulty,
                                Events,
                                TimingPoints,
                                Colours,
                                HitObjects,
                            }
                            let mut parse_state = ParseState::None;
                            let mut lines = BufReader::new(file).lines();
                            if matches!(lines.next(), Some(line) if matches!(&line, Ok(line) if line.trim() == "osu file format v14")) {
                                for line in lines {
                                    let line = line.unwrap();
                                    let line= line.trim();
                                    match line {
                                        "" => {},
                                        _ if line.starts_with("//") => {},
                                        "[General]" => parse_state = ParseState::General,
                                        "[Editor]" => parse_state = ParseState::Editor,
                                        "[Metadata]" => parse_state = ParseState::Metadata,
                                        "[Difficulty]" => parse_state = ParseState::Difficulty,
                                        "[Events]" => parse_state = ParseState::Events,
                                        "[TimingPoints]" => parse_state = ParseState::TimingPoints,
                                        "[Colours]" => parse_state = ParseState::Colours,
                                        "[HitObjects]" => parse_state = ParseState::HitObjects,
                                        _ => match parse_state {
                                            ParseState::None => {
                                                println!("Unknown tag: {}", line);
                                                valid = false;
                                                break;
                                            },
                                            ParseState::General => {
                                                let parts: Vec<&str> = line.split(": ").collect();
                                                if parts.len() != 2 {
                                                    eprintln!("Lines in general section must constist of exactly 2 pairs.");
                                                    valid = false;
                                                    break;
                                                }
                                                let key = *parts.get(0).unwrap();
                                                let value = *parts.get(1).unwrap();
                                                
                                                match key {
                                                    "AudioFilename" => audio_file_name = Some(value.to_string()),
                                                    "AudioLeadIn" => audio_lead_in = value.parse().unwrap(),
                                                    "Mode" => {
                                                        if value != "3" {
                                                            eprintln!("Only osu!mania maps are supported.");
                                                            valid = false;
                                                            break;
                                                        } else {
                                                            mode = Some(3);
                                                        }
                                                    },
                                                    "SpecialStyle" => {
                                                        if value != "0" {
                                                            eprintln!("Only non-special style is supported.");
                                                            valid = false;
                                                            break;
                                                        }
                                                    },
                                                    // Ignored keys
                                                    "AudioHash" |
                                                    "PreviewTime" |
                                                    "Countdown" |
                                                    "SampleSet" |
                                                    "StackLeniency" |
                                                    "LetterboxInBreaks" |
                                                    "StoryFireInFront" |
                                                    "UseSkipSprites" |
                                                    "AlwaysShowPlayfield" |
                                                    "OverlayPosition" |
                                                    "SkinPreference" |
                                                    "EpilepsyWarning" |
                                                    "CountdownOffset" |
                                                    "WidescreenStoryboard" |
                                                    "SampleMatchPlaybackRate" => {},
                                                    _ => eprintln!("Unrecognized key in General section: {}", key)
                                                }
                                            },
                                            ParseState::Editor => {
                                                let parts = line.split_once(": ");
                                                match parts {
                                                    Some((key, _value)) => {
                                                        match key {
                                                            // Ignored keys
                                                            "Bookmarks" |
                                                            "DistanceSpacing" |
                                                            "BeatDivisor" |
                                                            "GridSize" |
                                                            "TimelineZoom" => {},
                                                            _ => eprintln!("Unrecognized key in Editor section: {}", key)
                                                        }
                                                    },
                                                    None => {
                                                        eprintln!("Lines in Editor section must constist of exactly 2 pairs.");
                                                        valid = false;
                                                        break;
                                                    }
                                                }
                                            },
                                            ParseState::Metadata => {
                                                let parts = line.split_once(":");
                                                match parts {
                                                    Some((key, value)) => {
                                                        match key {
                                                            "TitleUnicode" => title = Some(value.to_string()),
                                                            "Version" => version = Some(value.to_owned()),
                                                            // Ignored keys
                                                            "Title" |
                                                            "Artist" |
                                                            "ArtistUnicode" |
                                                            "Creator" |
                                                            "Source" |
                                                            "Tags" |
                                                            "BeatmapID" |
                                                            "BeatmapSetID" => {},
                                                            _ => eprintln!("Unrecognized key in Metadata section: {}", key)
                                                        }
                                                    },
                                                    None => {
                                                        eprintln!("Lines in Metadata section must constist of exactly 2 pairs. {}", line);
                                                        valid = false;
                                                        break;
                                                    }
                                                }
                                            },
                                            ParseState::Difficulty => {
                                                let parts: Vec<&str> = line.split(":").collect();
                                                if parts.len() != 2 {
                                                    eprintln!("Lines in difficulty section must constist of exactly 2 pairs.");
                                                    valid = false;
                                                    break;
                                                }
                                                let key = *parts.get(0).unwrap();
                                                let value = *parts.get(1).unwrap();
                                                
                                                match key {
                                                    "CircleSize" => if value == "4" {
                                                        column_count = Some(4);
                                                    } else {
                                                        eprintln!("Only 4k is supported.");
                                                        valid = false;
                                                        break;
                                                    }
                                                    // Ignored keys
                                                    "HPDrainRate" |
                                                    "OverallDifficulty" |
                                                    "ApproachRate" |
                                                    "SliderMultiplier" |
                                                    "SliderTickRate" => {},
                                                    _ => eprintln!("Unrecognized key in Difficulty section: {}", key)
                                                }
                                            },
                                            ParseState::Events => {},
                                            ParseState::TimingPoints => {},
                                            ParseState::Colours => {},
                                            ParseState::HitObjects => {
                                                // Assumes osu!mania
                                                enum HitObjectParseState {
                                                    X,
                                                    Y,
                                                    Time,
                                                    Type,
                                                }
                                                let mut state = HitObjectParseState::X;
                                                let mut object_valid = true;
                                                let mut x: Option<usize> = None;
                                                let mut time: Option<usize> = None;
                                                for part in line.split(",") {
                                                    match state {
                                                        HitObjectParseState::X => {
                                                            x = Some(part.parse::<usize>().unwrap());
                                                            state = HitObjectParseState::Y;
                                                        },

                                                        HitObjectParseState::Y => state = HitObjectParseState::Time,
                                                        HitObjectParseState::Time => {
                                                            time = Some(part.parse::<usize>().unwrap());
                                                            state = HitObjectParseState::Type;
                                                        },
                                                        HitObjectParseState::Type => if part != "1" {
                                                            eprintln!("Only hit objects of type hit circle (1) are supported.");
                                                            object_valid = false;
                                                            break;
                                                        } else {
                                                            break; // Rest aren't read for now
                                                        }
                                                    }
                                                }

                                                if object_valid {
                                                    if let Some(x) = x {
                                                        if let Some(time) = time {
                                                            hit_circles.push(HitCircle {
                                                                x,
                                                                time,
                                                            });
                                                        } else {
                                                            eprintln!("Hit object does not have a time value. Ignoring...");
                                                        }
                                                    } else {
                                                        eprintln!("Hit object does not have an x value. Ignoring...");
                                                    }
                                                } else {
                                                    eprintln!("Hit object is invalid. Ignoring...");
                                                }
                                            }
                                        }
                                    }
                                }
                            } else {
                                valid = false;
                            }

                            if valid {
                                if let Some(audio_file_name) = audio_file_name {
                                    if mode == Some(3) {
                                        if let Some(title) = title {
                                            if let Some(version) = version {
                                                if let Some(column_count) = column_count {
                                                    let mut notes = [Vec::new(), Vec::new(), Vec::new(), Vec::new()];

                                                    for hit_circle in hit_circles {
                                                        let column = hit_circle.x*column_count/512;
                                                        if column < column_count {
                                                            notes[column].push(hit_circle.time);
                                                        } else {
                                                            eprintln!("Hit Circle does not fit in columns. Ignoring...");
                                                        }
                                                    }

                                                    maps.push(Map {
                                                        audio_file_name,
                                                        audio_lead_in,
                                                        full_title: format!("{} - {}", version, title),
                                                        notes
                                                    });
                                                } else {
                                                    eprintln!("Map does not report its circle size (also column count). Ignoring...");
                                                }
                                            } else {
                                                eprintln!("Map does not report a version. Ignoring...");
                                            }
                                        } else {
                                            eprintln!("Map does not have a title. Ignoring...");
                                        }
                                    } else {
                                        eprintln!("Map does not report to be osu!mania. Ignoring...");
                                    }
                                } else {
                                    eprintln!("Map does not have an audio file listed. Ignoring...");
                                }
                            } else {
                                eprintln!("Map is invalid. Ignoring...");
                            }
                        }
                },
                _ => {
                    {
                        let name = file.name().to_string();
                        eprintln!("Reading file {:#?}...", name);
                        let mut bytes = Vec::new();
                        file.read_to_end(&mut bytes).expect(format!("failed to read file: {}", name).as_str());
                        files.insert(name, bytes);
                    }
                }
            }
        }

        Set {
            maps,
            files,
        }
    }
}