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

  // Función para hacer la llamada a la API y cambiar el estado del firewall
  const toggleFirewall = async () => {
    setLoading(true); // Mostrar el estado de carga
    try {
      // TODO: IMPLEMENT API CALL TO TURN API ON
      const response = await fetch('https://api.example.com/firewall', {
        method: 'POST', // O 'PUT', dependiendo de la API
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          action: firewallActive ? 'turn_off' : 'turn_on',
        }),
      });

      if (!response.ok) {
        throw new Error('Error al comunicarse con la API');
      }

      const result = await response.json();

      // Actualiza el estado solo si la respuesta de la API es correcta
      if (result.success) {
        setFirewallActive(!firewallActive); // Cambia el estado del firewall
      } else {
        console.error('Error en la respuesta de la API');
      }
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
        disabled={loading} // Desactivar el botón mientras está en proceso
      >
        {loading ? 'Processing...' : firewallActive ? 'Turn Off' : 'Turn On'}
      </button>
      <Link to="/">Back to Home</Link>
    </div>
  );
}

export default App;
