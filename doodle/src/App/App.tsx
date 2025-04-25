import "./App.css";

const personalInfo = {
  name: "Austin Keisling",
  title: "Software Engineer",
  bio: "I'm a passionate technologist who loves building web applications, automating tasks, and solving problems.",
  github: "https://github.com/akeisling-ncino",
  linkedin: "https://linkedin.com/in/austin-a-keisling",
};

const repositories = [
  {
    name: "fictional-doodle",
    description:
      "A simple web application built with React, TypeScript, and Vite",
    url: "https://github.com/akeisling-ncino/fictional-doodle",
    topics: ["react", "typescript", "vite"],
  },
  {
    name: "force-nComplianceUS",
    description: "A Salesforce app for managing compliance",
    url: "https://github.com/ncino/force-nComplianceUS",
    topics: ["salesforce", "javascript", "apex"],
  },
  {
    name: "force-nSBA",
    description: "ETran integration for SBA",
    url: "https://github.com/ncino/force-nSBA",
    topics: ["salesforce", "javascript", "apex"],
  },
  {
    name: "force-customer-onboarding",
    description: "A Salesforce app for managing customer onboarding",
    url: "https://github.com/ncino/force-customer-onboarding",
    topics: ["salesforce", "javascript", "apex"],
  },
  {
    name: "aws-business-banking",
    description: "An AWS Lambda function for business banking",
    url: "https://github.com/ncino/aws-business-banking",
    topics: ["aws", "lambda", "typescript"],
  },
  {
    name: "aws-omni-channel",
    description: "A Ruby on Rails app for omni-channel",
    url: "https://github.com/ncino/aws-omni-channel",
    topics: ["ruby", "vue", "typescript"],
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
            href={personalInfo.github}
            target="_blank"
            rel="noopener noreferrer"
          >
            GitHub
          </a>
          <a
            href={personalInfo.linkedin}
            target="_blank"
            rel="noopener noreferrer"
          >
            LinkedIn
          </a>
        </div>
      </section>

      <section className="repositories">
        <h2>My Projects</h2>
        <div className="repo-grid">
          {repositories.map((repo) => (
            <div className="repo-card" key={repo.name}>
              <h3>{repo.name}</h3>
              <p>{repo.description}</p>
              <div className="topics">
                {repo.topics.map((topic) => (
                  <span className="topic" key={topic}>
                    {topic}
                  </span>
                ))}
              </div>
              <a href={repo.url} target="_blank" rel="noopener noreferrer">
                View on GitHub
              </a>
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
