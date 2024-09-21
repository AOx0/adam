import React, { useState } from 'react';

function FirewallRules({ firewallRules, loadingRules, toggleExpand, isExpanded }) {
  return (
    <div className="expander-section">
      <button
        onClick={toggleExpand}
        className="expander-btn"
      >
        {isExpanded ? 'Hide Rules' : 'Show Rules'}
      </button>
      <div id="contenido">
        {isExpanded && (
          <div className="rules-list" id="first">
            <h3>Firewall Rules</h3>
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
                  {firewallRules.map((rule, number) => (
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
      </div>
    </div>
  );
}

export default FirewallRules;
