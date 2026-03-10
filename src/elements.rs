/// ZZT element types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Element {
    Empty = 0,
    BoardEdge = 1,
    Messenger = 2,
    Monitor = 3,
    Player = 4,
    Ammo = 5,
    Torch = 6,
    Gem = 7,
    Key = 8,
    Door = 9,
    Scroll = 10,
    Passage = 11,
    Duplicator = 12,
    Bomb = 13,
    Energizer = 14,
    Star = 15,
    Clockwise = 16,
    Counter = 17,
    Bullet = 18,
    Water = 19,
    Forest = 20,
    Solid = 21,
    Normal = 22,
    Breakable = 23,
    Boulder = 24,
    SliderNs = 25,
    SliderEw = 26,
    Fake = 27,
    Invisible = 28,
    BlinkWall = 29,
    Transporter = 30,
    Line = 31,
    Ricochet = 32,
    BlinkRayH = 33,
    Bear = 34,
    Ruffian = 35,
    Object = 36,
    Slime = 37,
    Shark = 38,
    SpinningGun = 39,
    Pusher = 40,
    Lion = 41,
    Tiger = 42,
    BlinkRayV = 43,
    Head = 44,
    Segment = 45,
    // 46 is unused
    TextBlue = 47,
    TextGreen = 48,
    TextCyan = 49,
    TextRed = 50,
    TextPurple = 51,
    TextBrown = 52,
    TextBlack = 53,
}

impl Element {
    /// Try to convert a u8 to an Element.
    pub fn from_u8(id: u8) -> Option<Element> {
        match id {
            0 => Some(Element::Empty),
            1 => Some(Element::BoardEdge),
            2 => Some(Element::Messenger),
            3 => Some(Element::Monitor),
            4 => Some(Element::Player),
            5 => Some(Element::Ammo),
            6 => Some(Element::Torch),
            7 => Some(Element::Gem),
            8 => Some(Element::Key),
            9 => Some(Element::Door),
            10 => Some(Element::Scroll),
            11 => Some(Element::Passage),
            12 => Some(Element::Duplicator),
            13 => Some(Element::Bomb),
            14 => Some(Element::Energizer),
            15 => Some(Element::Star),
            16 => Some(Element::Clockwise),
            17 => Some(Element::Counter),
            18 => Some(Element::Bullet),
            19 => Some(Element::Water),
            20 => Some(Element::Forest),
            21 => Some(Element::Solid),
            22 => Some(Element::Normal),
            23 => Some(Element::Breakable),
            24 => Some(Element::Boulder),
            25 => Some(Element::SliderNs),
            26 => Some(Element::SliderEw),
            27 => Some(Element::Fake),
            28 => Some(Element::Invisible),
            29 => Some(Element::BlinkWall),
            30 => Some(Element::Transporter),
            31 => Some(Element::Line),
            32 => Some(Element::Ricochet),
            33 => Some(Element::BlinkRayH),
            34 => Some(Element::Bear),
            35 => Some(Element::Ruffian),
            36 => Some(Element::Object),
            37 => Some(Element::Slime),
            38 => Some(Element::Shark),
            39 => Some(Element::SpinningGun),
            40 => Some(Element::Pusher),
            41 => Some(Element::Lion),
            42 => Some(Element::Tiger),
            43 => Some(Element::BlinkRayV),
            44 => Some(Element::Head),
            45 => Some(Element::Segment),
            47 => Some(Element::TextBlue),
            48 => Some(Element::TextGreen),
            49 => Some(Element::TextCyan),
            50 => Some(Element::TextRed),
            51 => Some(Element::TextPurple),
            52 => Some(Element::TextBrown),
            53 => Some(Element::TextBlack),
            _ => None,
        }
    }

    /// Get the human-readable name for this element.
    pub fn name(self) -> &'static str {
        match self {
            Element::Empty => "empty",
            Element::BoardEdge => "board_edge",
            Element::Messenger => "messenger",
            Element::Monitor => "monitor",
            Element::Player => "player",
            Element::Ammo => "ammo",
            Element::Torch => "torch",
            Element::Gem => "gem",
            Element::Key => "key",
            Element::Door => "door",
            Element::Scroll => "scroll",
            Element::Passage => "passage",
            Element::Duplicator => "duplicator",
            Element::Bomb => "bomb",
            Element::Energizer => "energizer",
            Element::Star => "star",
            Element::Clockwise => "clockwise",
            Element::Counter => "counter",
            Element::Bullet => "bullet",
            Element::Water => "water",
            Element::Forest => "forest",
            Element::Solid => "solid",
            Element::Normal => "normal",
            Element::Breakable => "breakable",
            Element::Boulder => "boulder",
            Element::SliderNs => "sliderns",
            Element::SliderEw => "sliderew",
            Element::Fake => "fake",
            Element::Invisible => "invisible",
            Element::BlinkWall => "blinkwall",
            Element::Transporter => "transporter",
            Element::Line => "line",
            Element::Ricochet => "ricochet",
            Element::BlinkRayH => "blink_ray_h",
            Element::Bear => "bear",
            Element::Ruffian => "ruffian",
            Element::Object => "object",
            Element::Slime => "slime",
            Element::Shark => "shark",
            Element::SpinningGun => "spinninggun",
            Element::Pusher => "pusher",
            Element::Lion => "lion",
            Element::Tiger => "tiger",
            Element::BlinkRayV => "blink_ray_v",
            Element::Head => "head",
            Element::Segment => "segment",
            Element::TextBlue => "text_blue",
            Element::TextGreen => "text_green",
            Element::TextCyan => "text_cyan",
            Element::TextRed => "text_red",
            Element::TextPurple => "text_purple",
            Element::TextBrown => "text_brown",
            Element::TextBlack => "text_black",
        }
    }

    /// Get the alias for parameter 1 based on element type.
    pub fn p1_alias(self) -> Option<&'static str> {
        match self {
            Element::BlinkWall => Some("start_time"),
            Element::Bear => Some("sensitivity"),
            Element::Ruffian => Some("intelligence"),
            Element::Object => Some("char"),
            Element::Shark => Some("intelligence"),
            Element::SpinningGun => Some("intelligence"),
            Element::Lion => Some("intelligence"),
            Element::Tiger => Some("intelligence"),
            Element::Head => Some("intelligence"),
            _ => None,
        }
    }

    /// Get the alias for parameter 2 based on element type.
    pub fn p2_alias(self) -> Option<&'static str> {
        match self {
            Element::Duplicator => Some("rate"),
            Element::BlinkWall => Some("period"),
            Element::Ruffian => Some("resting_time"),
            Element::Slime => Some("speed"),
            Element::SpinningGun => Some("firing_rate"),
            Element::Tiger => Some("firing_rate"),
            Element::Head => Some("deviance"),
            _ => None,
        }
    }

    /// Get the alias for parameter 3 based on element type.
    pub fn p3_alias(self) -> Option<&'static str> {
        match self {
            Element::Passage => Some("destination"),
            Element::SpinningGun => Some("firing_type"),
            Element::Tiger => Some("firing_type"),
            _ => None,
        }
    }

    /// Map alias name to parameter number (1, 2, or 3) based on element type.
    pub fn alias_to_param(self, alias: &str) -> Option<u8> {
        if self.p1_alias() == Some(alias) {
            Some(1)
        } else if self.p2_alias() == Some(alias) {
            Some(2)
        } else if self.p3_alias() == Some(alias) {
            Some(3)
        } else {
            None
        }
    }
}

/// Resolve alias name to parameter number, with optional element context.
/// Falls back to checking all known aliases if element is None.
pub fn resolve_alias(alias: &str, element: Option<Element>) -> Option<u8> {
    // If we have an element, use its specific alias mapping
    if let Some(elem) = element {
        if let Some(param) = elem.alias_to_param(alias) {
            return Some(param);
        }
    }

    // Fallback: check all known aliases
    // p1 aliases
    if matches!(
        alias,
        "char" | "sensitivity" | "intelligence" | "start_time"
    ) {
        return Some(1);
    }

    // p2 aliases
    if matches!(
        alias,
        "rate" | "period" | "resting_time" | "speed" | "firing_rate" | "deviance"
    ) {
        return Some(2);
    }

    // p3 aliases
    if matches!(alias, "destination" | "firing_type") {
        return Some(3);
    }

    None
}

/// Get the human-readable name for an element ID, or describe unknown elements.
pub fn element_name(id: u8) -> String {
    match Element::from_u8(id) {
        Some(e) => e.name().to_string(),
        None => format!("unknown ({})", id),
    }
}

/// Convert an element name back to its ID.
/// Handles "unknown_N" format for unknown element IDs.
pub fn element_id_from_name(name: &str) -> Option<u8> {
    // Handle "unknown_N" format first
    if let Some(suffix) = name.strip_prefix("unknown_") {
        return suffix.parse().ok();
    }
    // Match all known element names to their IDs
    match name {
        "empty" => Some(0),
        "board_edge" => Some(1),
        "messenger" => Some(2),
        "monitor" => Some(3),
        "player" => Some(4),
        "ammo" => Some(5),
        "torch" => Some(6),
        "gem" => Some(7),
        "key" => Some(8),
        "door" => Some(9),
        "scroll" => Some(10),
        "passage" => Some(11),
        "duplicator" => Some(12),
        "bomb" => Some(13),
        "energizer" => Some(14),
        "star" => Some(15),
        "clockwise" => Some(16),
        "counter" => Some(17),
        "bullet" => Some(18),
        "water" => Some(19),
        "forest" => Some(20),
        "solid" => Some(21),
        "normal" => Some(22),
        "breakable" => Some(23),
        "boulder" => Some(24),
        "sliderns" => Some(25),
        "sliderew" => Some(26),
        "fake" => Some(27),
        "invisible" => Some(28),
        "blinkwall" => Some(29),
        "transporter" => Some(30),
        "line" => Some(31),
        "ricochet" => Some(32),
        "blink_ray_h" => Some(33),
        "bear" => Some(34),
        "ruffian" => Some(35),
        "object" => Some(36),
        "slime" => Some(37),
        "shark" => Some(38),
        "spinninggun" => Some(39),
        "pusher" => Some(40),
        "lion" => Some(41),
        "tiger" => Some(42),
        "blink_ray_v" => Some(43),
        "head" => Some(44),
        "segment" => Some(45),
        // 46 is unused
        "text_blue" => Some(47),
        "text_green" => Some(48),
        "text_cyan" => Some(49),
        "text_red" => Some(50),
        "text_purple" => Some(51),
        "text_brown" => Some(52),
        "text_black" => Some(53),
        _ => None,
    }
}
