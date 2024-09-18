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
  const [firewallActive, setFirewallActive] = useState(true); // Estado del firewall
  const [loading, setLoading] = useState(false); // Estado de carga
  const [isExpanded, setIsExpanded] = useState(false); // Estado del expander

  // Lista de eventos simulados
  const events = [
    { id: 1, name: 'Firewall Enabled', description: 'The firewall was enabled on 2024-09-17 at 10:00 AM' },
    { id: 2, name: 'Firewall Disabled', description: 'The firewall was disabled on 2024-09-16 at 5:00 PM' },
    { id: 3, name: 'Intrusion Detected', description: 'An intrusion attempt was detected on 2024-09-15 at 2:00 AM' }
  ];

  // Función para alternar el estado del expander
  const toggleExpand = () => {
    setIsExpanded(!isExpanded);
  };

    // Función para hacer la llamada a la API y cambiar el estado del firewall
    const toggleFirewall = async () => {
      setLoading(true); // Mostrar el estado de carga
      

      const url = firewallActive 
        ? 'http://201.121.247.43:80/firewall/halt' // Apagar firewall
        : 'http://201.121.247.43:80/firewall/start'; // Encender firewall
    
       console.log(`Sending request to: ${url}`);

       try {
        // Realiza la solicitud con no-cors
        await fetch(url, {
          method: 'POST',
          mode: 'no-cors', // Añadir el modo no-cors
          headers: {
            'Content-Type': 'application/json',
          },
        });
  
      // Si no hay error, asume que la solicitud fue exitosa
         console.log('Request sent with no-cors mode.');
         
      

      // Cambia el estado del firewall sin depender de la respuesta de la API
      setFirewallActive(!firewallActive); // Cambia el estado de activo a inactivo o viceversa
      console.log(`Firewall is now ${!firewallActive ? 'active' : 'inactive'}`);
    } catch (error) {
      console.error('Error al realizar la solicitud:', error);
    } finally {
      setLoading(false); // Termina el estado de carga
    }
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
          className="toggle-firewall-btn"
          disabled={loading}
          style={{ 
            backgroundColor: firewallActive ? 'red' : 'green', // Cambia el color del botón
            color: 'white' // Color del texto del botón
          }}
        >
          {loading ? 'Processing...' : firewallActive ? 'Turn Off' : 'Turn On'}
        </button>

      {/* Botón de mostrar eventos */}
      <div className="expander-section">
        <button
          onClick={toggleExpand}
          className="expander-btn"
        >
          {isExpanded ? 'Hide Rules' : 'Show Rules'}
        </button>

        {/* Mostrar la lista de eventos si el expander está abierto */}
        {isExpanded && (
          <div className="rule-list">
            <h3>Firewall Rules</h3>

            {/* Muestra un indicador de carga mientras se obtienen las reglas */}
            {loadingRules ? (
              <p>Loading rules...</p>
            ) : (
              <table>
                <thead>
                  <tr>
                    <th>Action</th>
                    <th>Protocol</th>
                    <th>Applies To</th>
                    <th>Enabled</th>
                    <th>Init</th>
                  </tr>
                </thead>
                <tbody>
                  {firewallRules.map((rule, index) => (
                    <tr key={index}>
                      <td>{rule.action}</td>
                      <td>{rule.matches.Protocol}</td>
                      <td>{rule.applies_to}</td>
                      <td>{rule.enabled ? 'Yes' : 'No'}</td>
                      <td>{rule.init ? 'Yes' : 'No'}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            )}
          </div>
        )}
      </div>

      {/* Link para volver a Home */}
      <Link to="/" style={{ marginTop: '20px', display: 'block' }}>Back to Home</Link>
    </div>
  );
}

export default App;
