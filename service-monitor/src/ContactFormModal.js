import React, { useState } from 'react';
import Modal from './Modal';


function ContactFormModal() {
  const [isModalOpen, setIsModalOpen] = useState(false);

  const handleSubmit = (e) => {
    e.preventDefault();
    console.log("Formulario enviado");
    setIsModalOpen(false); // Cerrar el modal después de enviar el formulario
  };

  return (
    <div>
      <button onClick={() => setIsModalOpen(true)} className="open-modal-btn">
        Open Form
      </button>

      <Modal isOpen={isModalOpen} onClose={() => setIsModalOpen(false)}>
        <h2>Add Rule</h2>
        <form onSubmit={handleSubmit}>
          <div className="form-group">
            <label htmlFor="name">Nombre:</label>
            <input type="text" id="name" name="name" required />
          </div>
          <div className="form-group">
            <label htmlFor="email">Correo:</label>
            <input type="email" id="email" name="email" required />
          </div>
          <div className="form-group">
            <label htmlFor="message">Mensaje:</label>
            <textarea id="message" name="message" required></textarea>
          </div>
          <button type="submit" className="submit-btn">Enviar</button>
        </form>
      </Modal>
    </div>
  );
}

export default ContactFormModal;
