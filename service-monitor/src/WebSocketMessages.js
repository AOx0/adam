import React, { useState, useEffect } from 'react';

function WebSocketMessages() {
  const [webSocketMessages, setWebSocketMessages] = useState([]);

  useEffect(() => {
    const ws = new WebSocket('ws:/201.121.247.43:80/firewall/events');

    ws.onopen = () => {
      console.log('Conexión WebSocket establecida');
      ws.send('Cliente conectado');
    };

    ws.onmessage = (event) => {
      setWebSocketMessages((prevMessages) => {
        const newMessages = [...prevMessages, event.data];
        return newMessages.slice(-50); // Limitar a los últimos 50 mensajes
      });
    };

    ws.onclose = () => {
      console.log('Conexión WebSocket cerrada');
    };

    return () => {
      ws.close();
    };
  }, []);

  return (
    <div className="websocket-messages">
      <h3>WebSocket Messages</h3>
      <div className="scrollable-window">
        {webSocketMessages.map((msg, index) => (
          <div key={index} className="message">
            {msg}
          </div>
        ))}
      </div>
    </div>
  );
}

export default WebSocketMessages;
