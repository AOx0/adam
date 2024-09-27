import React from 'react'

export const logo = (
  <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 100" width="400" height="200">
    {/* Fondo */}
    <rect width="100%" height="100%" fill="#f0f8ff" />

    {/* Texto "Adam" */}
    <text x="20" y="60" fontFamily="Arial" fontSize="40" fill="#333">
      Adam
    </text>

    {/* Abeja estilizada */}
    {/* Cuerpo de la abeja */}
    <ellipse cx="150" cy="50" rx="20" ry="12" fill="#ffcc00" stroke="#000" strokeWidth="2" />

    {/* Rayas negras de la abeja */}
    <line x1="138" y1="50" x2="162" y2="50" stroke="#000" strokeWidth="2" />
    <line x1="145" y1="45" x2="155" y2="45" stroke="#000" strokeWidth="2" />
    <line x1="145" y1="55" x2="155" y2="55" stroke="#000" strokeWidth="2" />

    {/* Cabeza de la abeja */}
    <circle cx="130" cy="50" r="7" fill="#000" />

    {/* Alas de la abeja */}
    <ellipse cx="150" cy="40" rx="10" ry="5" fill="#b3e6ff" stroke="#000" strokeWidth="1" />
    <ellipse cx="150" cy="60" rx="10" ry="5" fill="#b3e6ff" stroke="#000" strokeWidth="1" />

    {/* Antenas */}
    <line x1="126" y1="45" x2="120" y2="40" stroke="#000" strokeWidth="1.5" />
    <line x1="134" y1="45" x2="140" y2="40" stroke="#000" strokeWidth="1.5" />
  </svg>
)
