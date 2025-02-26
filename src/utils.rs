use rand::Rng;

pub fn roll(chance: i8) -> bool {
    rand::thread_rng().gen_range(0..100) < chance
}

pub fn roll_range(min_chance: i8, max_chance: i8) -> bool {
    let chance = rand::thread_rng().gen_range(min_chance..max_chance);
    roll(chance)
}

pub fn random_precept() -> &'static str {
    let choice = rand::thread_rng().gen_range(0..PRECEPTS.len()); // Fix range
    PRECEPTS[choice]
}

pub const PRECEPTS: [&str; 18] = [
    "Accept everything just the way it is.",
    "Do not seek pleasure for its own sake.",
    "Do not, under any circumstances, depend on a partial feeling.",
    "Think lightly about yourself and deeply about the world.",
    "Be detached from desire your whole life long.",
    "Do not regret what you have done.",
    "Never be jealous.",
    "Never let yourself be saddened by separation.",
    "Resentment and complaint are appropriate neither for oneself nor others.",
    "Do not let yourself be guided by the feeling of lust or love.",
    "In all things have no preferences.",
    "Be indifferent to where you live.",
    "Do not pursue the taste of good food.",
    "Do not hold on to possessions you no longer need.",
    "Do not act following customary beliefs.",
    "Do not collect weapons or practice with weapons beyond what is useful.",
    "Do not fear death.",
    "Do not seek to possess either goods or fiefs for your old age.",
];
