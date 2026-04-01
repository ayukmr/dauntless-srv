import React, { Component } from 'react';

const Context = React.createContext(null);

class Provider extends Component {
  state = {
    id: 0,
    data: null,
    meta: null,
    config: null,
    errors: {}
  };

  componentDidMount() {
    this.fetchMeta();
    this.fetchConfig();
    this.connectWS();
  }

  componentWillUnmount() {
    super.componentWillUnmount();
    this.disconnectWS();
  }

  componentDidUpdate(prev) {
    if (prev.id !== this.props.id) {
      this.disconnectWS();
      this.connectWS();
    }
  }

  connectWS = () => {
    const url = `/api/${this.state.id}/data`;
    this.ws = new WebSocket(url);

    this.ws.onmessage = (event) => {
      const data = JSON.parse(event.data);
      this.setState({ data });
      this.updateError(url, false);
    };

    this.ws.onerror = () => {
      this.updateError(url, true);
    };
  };

  disconnectWS = () => {
    if (!this.ws) return;

    this.ws.onclose = null;
    this.ws.onerror = null;
    this.ws.onmessage = null;

    this.ws.close();
    this.ws = null;
  };

  isLoaded = () => {
    const { data, meta, config } = this.state;
    return data !== null && meta !== null && config !== null;
  };

  isConnected = () => {
    return !Object.values(this.state.errors).some(Boolean);
  };

  update = (domain, settings) => {
    this.setState((prev) => {
      const updated = {
        ...prev.config,
        [domain]: {
          ...prev.config[domain],
          ...settings
        }
      };

      return { config: updated };
    }, () => this.setConfig());
  };

  updateID = (id) => {
    this.setState({ id }, () => this.fetchConfig());
  };

  updateError = (key, on) => {
    this.setState((prev) => ({
      errors: {
        ...prev.errors,
        [key]: on
      }
    }));
  };

  fetch = async (url, key) => {
    try {
      const res = await fetch(url);
      this.setState({ [key]: await res.json() });
      this.updateError(url, false);
    } catch {
      this.updateError(url, true);
    }
  };

  fetchMeta = async () => {
    await this.fetch('/api/meta', 'meta');
  };

  fetchConfig = async () => {
    await this.fetch(`/api/${this.state.id}/config`, 'config');
  };

  setConfig = async () => {
    await fetch(`/api/${this.state.id}/config`, {
      method: 'POST',
      body: JSON.stringify(this.state.config),
    });
  };

  render() {
    return (
      <Context.Provider
        value={{
          isLoaded: this.isLoaded,
          isConnected: this.isConnected,
          update: this.update,
          updateID: this.updateID,
          updateError: this.updateError,
          ...this.state,
        }}
      >
        {this.props.children}
      </Context.Provider>
    );
  }
}

export { Context, Provider };
