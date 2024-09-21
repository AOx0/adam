import React, { useState } from 'react';

function FirewallRules({ firewallRules, loadingRules, toggleExpand, isExpanded, onDeleteRule }) {
  const [selectedRuleId, setSelectedRuleId] = useState('');

  const handleDelete = () => {
    if (selectedRuleId) {
      onDeleteRule(selectedRuleId);
      setSelectedRuleId(''); // Resetear selección después de eliminar
    }
  };

  return (
    <div className="expander-section">
      <button onClick={toggleExpand} className="expander-btn">
        {isExpanded ? 'Hide Rules' : 'Show Rules'}
      </button>
      <div id="contenido">
        {isExpanded && (
          <div className="rules-list" id="first">
            <h3>Firewall Rules</h3>
            {loadingRules ? (
              <p>Loading rules...</p>
            ) : (
              <>
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
                    {firewallRules.map((rule) => (
                      <tr key={rule.id}>
                        <td>{rule.id}</td>
                        <td>{rule.action}</td>
                        <td>{JSON.stringify(rule.matches)}</td>
                        <td>{rule.applies_to}</td>
                        <td>{rule.enabled ? 'Yes' : 'No'}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </>
            )}
            <div className="delete-rule">
        <h4>Delete Rule</h4>
        <select value={selectedRuleId} onChange={(e) => setSelectedRuleId(e.target.value)}>
          <option value="">Select Rule ID</option>
          {firewallRules.map((rule) => (
            <option key={rule.id} value={rule.id}>{rule.id}</option>
          ))}
        </select>
        <button onClick={handleDelete} className="delete-btn">Delete Rule</button>
      </div>
          </div>
        )}
      
      
      
      </div>
    </div>
  );
}

export default FirewallRules;
