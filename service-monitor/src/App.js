import React, { useState, useEffect } from 'react';
import { BrowserRouter as Router, Route, Routes, Link } from 'react-router-dom';
import './App.css';
import ContactFormModal from './ContactFormModal';
import WebSocketMessages from './WebSocketMessages';
import FirewallRules from './FirewallRules';

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
  const [firewallActive, setFirewallActive] = useState(true);
  const [loading, setLoading] = useState(false);
  const [isExpanded, setIsExpanded] = useState(false);
  const [firewallRules, setFirewallRules] = useState([]);
  const [loadingRules, setLoadingRules] = useState(false);

  const toggleExpand = async () => {
    setIsExpanded(!isExpanded);
    if (!isExpanded) {
      setLoadingRules(true);
      try {
        const response = await fetch('http://172.29.34.232:80/firewall/rules');
        if (response.ok) {
          const data = await response.json();
          setFirewallRules(data);
        }
      } catch (error) {
        console.error('Error al obtener las reglas del firewall:', error);
      } finally {
        setLoadingRules(false);
      }
    }
  };

  const toggleFirewall = async () => {
    setLoading(true);
    const url = firewallActive 
      ? 'http:/172.29.34.232:80/firewall/halt'
      : 'http://172.29.34.232:80/firewall/start';
    try {
      await fetch(url, { method: 'POST', mode: 'no-cors' });
      setFirewallActive(!firewallActive);
    } catch (error) {
      console.error('Error al realizar la solicitud:', error);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="page">
      <h2>Firewall Monitor</h2>

      <p style={{ color: firewallActive ? 'green' : 'red' }}>
        Firewall is {firewallActive ? 'active' : 'inactive'}
      </p>

      <button
        onClick={toggleFirewall}
        className="toggle-firewall-btn"
        disabled={loading}
        style={{ 
          backgroundColor: firewallActive ? 'red' : 'green',
          color: 'white'
        }}
      >
        {loading ? 'Processing...' : firewallActive ? 'Turn Off' : 'Turn On'}
      </button>

      <ContactFormModal />
      <FirewallRules 
        firewallRules={firewallRules} 
        loadingRules={loadingRules}
        toggleExpand={toggleExpand}
        isExpanded={isExpanded}
      />
      <WebSocketMessages />
    </div>
  );
}

export default App;