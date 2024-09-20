import React, { useState, useEffect } from 'react';
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
  const [firewallRules, setFirewallRules] = useState([]); // Estado para almacenar las reglas del firewall
  const [loadingRules, setLoadingRules] = useState(false); // Estado para controlar la carga de las reglas
  const [webSocketMessages, setWebSocketMessages] = useState([]); // Estado para almacenar mensajes WebSocket


  // Función para alternar el estado del expander y obtener las reglas si se expande
  const toggleExpand = async () => {
    setIsExpanded(!isExpanded);

    if (!isExpanded) { // Solo hace la solicitud si se expande
      setLoadingRules(true); // Mostrar el estado de carga para las reglas
      try {
        const response = await fetch('http://172.29.34.232:80/firewall/rules', {
          method: 'GET',
        });

        if (response.ok) {
          const data = await response.json();
          setFirewallRules(data); // Almacenar las reglas obtenidas
        } else {
          console.error('Error al obtener las reglas del firewall');
        }
      } catch (error) {
        console.error('Error al realizar la solicitud:', error);
      } finally {
        setLoadingRules(false); // Finalizar el estado de carga para las reglas
      }
    }
  };

  // Función para hacer la llamada a la API y cambiar el estado del firewall
  const toggleFirewall = async () => {
    setLoading(true); // Mostrar el estado de carga

    const url = firewallActive 
      ? 'http:/172.29.34.232:80/firewall/halt' // Apagar firewall
      : 'http://172.29.34.232:80/firewall/start'; // Encender firewall

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

      // Cambia el estado del firewall sin depender de la respuesta de la API
      setFirewallActive(!firewallActive); // Cambia el estado de activo a inactivo o viceversa
      console.log(`Firewall is now ${!firewallActive ? 'active' : 'inactive'}`);
    } catch (error) {
      console.error('Error al realizar la solicitud:', error);
    } finally {
      setLoading(false); // Termina el estado de carga
    }
  };
  useEffect(() => {
    const ws = new WebSocket('ws:/172.29.34.232:80/firewall/events');

    // Abrir la conexión
    ws.onopen = () => {
      console.log('Conexión WebSocket establecida');
      ws.send('Cliente conectado');
    };

    // Recibir mensajes del WebSocket
    ws.onmessage = (event) => {
      console.log('Mensaje recibido desde el servidor:', event.data);
      setWebSocketMessages((prevMessages) => {
        const newMessages = [...prevMessages, event.data];
        return newMessages.slice(-50); // Limitar a los últimos 15 mensajes
      });
    };

    // Cerrar la conexión
    ws.onclose = () => {
      console.log('Conexión WebSocket cerrada');
    };

    // Limpiar conexión al desmontar el componente
    return () => {
      ws.close();
    };
  }, []);

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
        <div id="contenido">
        {/* Mostrar la lista de eventos si el expander está abierto */}
        {isExpanded && (
          <div className="rules-list" id="first">
            <h3>Firewall Rules</h3>

            {/* Muestra un indicador de carga mientras se obtienen las reglas */}
            {loadingRules ? (
              <p>Loading rules...</p>
            ) : (
              <table>
                <thead>
                  <tr>
                    <th>ID</th>
                    <th>Action</th>
                    <th>Rule</th>
                    <th>Applies To</th>
                    <th>Enabled</th>
                  </tr>
                </thead>
                <tbody>
                  {firewallRules.map((rule,number) => (
                    <tr key={number}>
                      <td>{rule.id}</td>
                      <td>{rule.action}</td>
                      <td>{JSON.stringify(rule.matches)}</td>
                      <td>{rule.applies_to}</td>
                      <td>{rule.enabled ? 'Yes' : 'No'}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            )}
          </div>
        )}
      </div >
      {/* Mostrar mensajes recibidos del WebSocket */}
      <div className="websocket-messages" id = "second">
        <h3>WebSocket Messages</h3>
        <ul>
          {webSocketMessages.map((msg, index) => (
            <li key={index}>{msg}</li>
          ))}
        </ul>
      </div>
      </div>
      {/*
      <div>
      <Link to="/" style={{ marginTop: '20px', display: 'block' }}>Back to Home</Link>
      </div>
       Link para volver a Home */}
    </div>
    
  );
}

export default App;
