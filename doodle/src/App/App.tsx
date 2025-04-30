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
    description: 
      "Korean agent who excels at mobility and quick strikes",
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
        <h2>Top Valorant Agents</h2>
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
