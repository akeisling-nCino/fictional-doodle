import "./App.css";

const personalInfo = {
  name: "Austin Keisling",
  title: "Valorant Enthusiast",
  bio: "I'm a passionate gamer who loves tactical shooters, especially Valorant. Here's my collection of favorite agents and their abilities.",
  twitter: "https://twitter.com/PlayVALORANT",
  valorant: "https://playvalorant.com/",
};

const agents = [
  {
    name: "Jett",
    description: "Korean agent who excels at mobility and quick strikes",
    role: "Duelist",
    abilities: ["Updraft", "Tailwind", "Cloudburst", "Blade Storm"],
  },
  {
    name: "Reyna",
    description: "Mexican agent who thrives on getting kills to heal and dismiss",
    role: "Duelist",
    abilities: ["Devour", "Dismiss", "Leer", "Empress"],
  },
  {
    name: "Chamber",
    description: "French weapons expert with precise gunplay and teleportation",
    role: "Sentinel",
    abilities: ["Headhunter", "Rendezvous", "Trademark", "Tour De Force"],
  },
  {
    name: "Killjoy",
    description: "German genius who secures areas with her inventions",
    role: "Sentinel",
    abilities: ["Alarmbot", "Turret", "Nanoswarm", "Lockdown"],
  },
  {
    name: "Omen",
    description: "Mysterious shadow who can teleport and blind enemies",
    role: "Controller",
    abilities: ["Paranoia", "Dark Cover", "Shrouded Step", "From the Shadows"],
  },
  {
    name: "Astra",
    description: "Ghanaian agent who harnesses the cosmos to reshape battlefields",
    role: "Controller",
    abilities: ["Nova Pulse", "Nebula", "Gravity Well", "Cosmic Divide"],
  },
  {
    name: "Breach",
    description: "Swedish initiator who uses seismic blasts to disable opponents",
    role: "Initiator",
    abilities: ["Flashpoint", "Fault Line", "Aftershock", "Rolling Thunder"],
  },
  {
    name: "Brimstone",
    description: "American commander with orbital tactical abilities",
    role: "Controller",
    abilities: ["Incendiary", "Sky Smoke", "Stim Beacon", "Orbital Strike"],
  },
  {
    name: "Cypher",
    description: "Moroccan surveillance expert who keeps tabs on the enemy's movements",
    role: "Sentinel",
    abilities: ["Cyber Cage", "Spycam", "Trapwire", "Neural Theft"],
  },
  {
    name: "Deadlock",
    description: "Norwegian sentinel who uses nanowire to create barriers",
    role: "Sentinel",
    abilities: ["Sonic Sensor", "Barrier Mesh", "GravNet", "Annihilation"],
  },
  {
    name: "Fade",
    description: "Turkish initiator who harnesses nightmares to reveal enemies",
    role: "Initiator",
    abilities: ["Haunt", "Seize", "Prowler", "Nightfall"],
  },
  {
    name: "Gekko",
    description: "Los Angeles native with creatures that help secure areas",
    role: "Initiator",
    abilities: ["Wingman", "Dizzy", "Mosh Pit", "Thrash"],
  },
  {
    name: "Harbor",
    description: "Indian controller who bends water to shield allies",
    role: "Controller",
    abilities: ["Cove", "High Tide", "Cascade", "Reckoning"],
  },
  {
    name: "ISO",
    description: "Chinese duelist who channels kinetic energy",
    role: "Duelist",
    abilities: ["Contingency", "Undercut", "Double Tap", "Kill Contract"],
  },
  {
    name: "KAY/O",
    description: "Robot initiator built for one thing: neutralizing radiants",
    role: "Initiator",
    abilities: ["FLASH/drive", "ZERO/point", "FRAG/ment", "NULL/cmd"],
  },
  {
    name: "Neon",
    description: "Filipino agent who races ahead to catch enemies off guard",
    role: "Duelist",
    abilities: ["High Gear", "Relay Bolt", "Fast Lane", "Overdrive"],
  },
  {
    name: "Phoenix",
    description: "British duelist who controls fire and can self-revive",
    role: "Duelist",
    abilities: ["Curveball", "Hot Hands", "Blaze", "Run It Back"],
  },
  {
    name: "Raze",
    description: "Brazilian demolition expert who causes explosive damage",
    role: "Duelist",
    abilities: ["Blast Pack", "Paint Shells", "Boom Bot", "Showstopper"],
  },
  {
    name: "Sage",
    description: "Chinese healer who can resurrect allies and create barriers",
    role: "Sentinel",
    abilities: ["Healing Orb", "Barrier Orb", "Slow Orb", "Resurrection"],
  },
  {
    name: "Skye",
    description: "Australian initiator who uses animal spirits to gather information",
    role: "Initiator",
    abilities: ["Trailblazer", "Guiding Light", "Regrowth", "Seekers"],
  },
  {
    name: "Sova",
    description: "Russian initiator who locates enemies with reconnaissance tools",
    role: "Initiator",
    abilities: ["Shock Bolt", "Recon Bolt", "Owl Drone", "Hunter's Fury"],
  },
  {
    name: "Viper",
    description: "American chemist who deploys poisonous gas and screens",
    role: "Controller",
    abilities: ["Snake Bite", "Poison Cloud", "Toxic Screen", "Viper's Pit"],
  },
  {
    name: "Yoru",
    description: "Japanese duelist who uses dimensional rifts to deceive enemies",
    role: "Duelist",
    abilities: ["Blindside", "Gatecrash", "Fakeout", "Dimensional Drift"],
  },
  {
    name: "Clove",
    description: "British controller who manipulates time to support team",
    role: "Controller",
    abilities: ["Meddle", "Pick-Me-Up", "Ruse", "Not Dead Yet"],
  },
];

function App() {
  return (
    <div className="container">
      <header>
        <h1>{personalInfo.name}</h1>
        <p className="title">{personalInfo.title}</p>
      </header>

      <section className="about">
        <h2>About Me</h2>
        <p>{personalInfo.bio}</p>
        <div className="social-links">
          <a
            href={personalInfo.twitter}
            target="_blank"
            rel="noopener noreferrer"
          >
            Twitter
          </a>
          <a
            href={personalInfo.valorant}
            target="_blank"
            rel="noopener noreferrer"
          >
            Play Valorant
          </a>
        </div>
      </section>

      <section className="agents">
        <h2>All Valorant Agents</h2>
        <div className="agent-grid">
          {agents.map((agent) => (
            <div className="agent-card" key={agent.name}>
              <h3>{agent.name}</h3>
              <p>{agent.description}</p>
              <div className="abilities">
                {agent.abilities.map((ability) => (
                  <span className="ability" key={ability}>
                    {ability}
                  </span>
                ))}
              </div>
              <span className="agent-role">Role: {agent.role}</span>
            </div>
          ))}
        </div>
      </section>

      <footer>
        <p>
          © {new Date().getFullYear()} {personalInfo.name}
        </p>
      </footer>
    </div>
  );
}

export default App;
