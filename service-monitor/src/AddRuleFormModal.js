import React, { useState } from 'react';
import Modal from './Modal';

function AddRuleFormModal() {
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [formData, setFormData] = useState({
    action: '', 
    protocol: '', 
    appliesTo: '', 
  });

  const handleChange = (e) => {
    const { name, value } = e.target;
    setFormData((prevData) => ({
      ...prevData,
      [name]: value,
    }));
  };

  const handleSubmit = async (e) => {
    e.preventDefault();
    
    const dataToSend = {
      action: formData.action, 
      matches: {
        protocol: formData.protocol, 
      },
      applies_to: formData.appliesTo, 
    };

    try {
      console.log(dataToSend);
      const response = await fetch('/firewall/add', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(dataToSend),
      });
  
      if (response.ok) {
        const data = await response.json();
        console.log('Rule added with ID:', data.id);
      } else {
        console.error('Error al agregar la regla:', response.statusText);
      }
    } catch (error) {
      console.error('Error al enviar el formulario:', error);
    } finally {
      setIsModalOpen(false);
    }
  };

  return (
    <div>
      <button onClick={() => setIsModalOpen(true)} className="open-modal-btn">
        Add Rule
      </button>

      <Modal isOpen={isModalOpen} onClose={() => setIsModalOpen(false)}>
        <h2>Add Rule</h2>
        <form onSubmit={handleSubmit}>
          <div className="form-group">
            <label htmlFor="action">Action:</label>
            <select id="action" name="action" value={formData.action} onChange={handleChange} required>
              <option value="">Select Action</option>
              <option value="drop">Drop</option>
              <option value="allow">Allow</option>
            </select>
          </div>
          <div className="form-group">
            <label htmlFor="protocol">Protocol:</label>
            <select id="protocol" name="protocol" value={formData.protocol} onChange={handleChange} required>
              <option value="">Select Protocol</option>
              <option value="ICMP">ICMP</option>
              <option value="TCP">TCP</option>
              <option value="UDP">UDP</option>
            </select>
          </div>
          <div className="form-group">
            <label htmlFor="appliesTo">Applies To:</label>
            <select id="appliesTo" name="appliesTo" value={formData.appliesTo} onChange={handleChange} required>
              <option value="">Select Applies To</option>
              <option value="source">Source</option>
              <option value="destination">Destination</option>
            </select>
          </div>
          <button type="submit" className="submit-btn">Enviar</button>
        </form>
      </Modal>
    </div>
  );
}

export default AddRuleFormModal;
