import React, { useState } from 'react';
import { BrowserRouter as Router, Route, Routes, Link } from 'react-router-dom';
import './App.css';

function App() {
  return (
    <Router>
      <div className="App">
        <nav className="App-navbar">
          <h1>ADAM Monitor</h1>
        </nav>
        <Routes>
          <Route path="/" element={<Home />} />
          <Route path="/firewall" element={<Page1 />} />
        </Routes>
      </div>
    </Router>
  );
}

function Home() {
  return (
    <header className="App-header">
      <div className="grid-container">
        <Link to="/firewall" className="grid-item">Firewall</Link>
        <div className="grid-item">2</div>
        <div className="grid-item">3</div>
        <div className="grid-item">4</div>
        <div className="grid-item">5</div>
        <div className="grid-item">6</div>
        <div className="grid-item">7</div>
        <div className="grid-item">8</div>
        <div className="grid-item">9</div>
      </div>
    </header>
  );
}

function Page1() {
  // Estado del firewall
  const [firewallActive, setFirewallActive] = useState(true);

  // Función para cambiar el estado del firewall
  const toggleFirewall = () => {
    setFirewallActive(!firewallActive);
  };

  return (
    <div className="page">
      <h2>Firewall Monitor</h2>
      {/* Mostrar el estado del firewall */}
      <p style={{ color: firewallActive ? 'green' : 'red' }}>
        Firewall is {firewallActive ? 'active' : 'inactive'}
      </p>
      {/* Botón de encendido/apagado */}
      <button
        onClick={toggleFirewall}
        style={{
          backgroundColor: firewallActive ? 'red' : 'green',
          color: 'white',
          padding: '10px 20px',
          border: 'none',
          borderRadius: '5px',
          cursor: 'pointer',
          position: 'absolute',
          top: '20px',
          right: '20px',
        }}
      >
        {firewallActive ? 'Turn Off' : 'Turn On'}
      </button>
      <Link to="/">Back to Home</Link>
    </div>
  );
}

export default App;
