import React, { Component } from 'react';

const Context = React.createContext(null);

class Provider extends Component {
  state = {
    data: null,
    meta: null,
    config: null,
    errors: {}
  };

  componentDidMount() {
    this.fetchMeta();
    this.fetchConfig();
    this.interval = setInterval(() => this.fetchData(), 50);
  }

  componentWillUnmount() {
    clearInterval(this.interval);
  }

  loaded = () => {
    const { data, meta, config } = this.state;
    return data !== null && meta !== null && config !== null;
  };

  connected = () => {
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

      this.setConfig(updated);
      return { config: updated };
    });
  };

  error = (key, on) => {
    this.setState((prev) => ({
      errors: {
        ...prev.errors,
        [key]: on
      }
    }));
  };

  fetch = async (key) => {
    const path = `/api/${key}`;

    try {
      const res = await fetch(path);
      this.setState({ [key]: await res.json() });
      this.error(path, false);
    } catch {
      this.error(path, true);
    }
  };

  fetchData = async () => {
    await this.fetch('data');
  };

  fetchMeta = async () => {
    await this.fetch('meta');
  };

  fetchConfig = async () => {
    await this.fetch('config');
  };

  setConfig = async (config) => {
    await fetch('/api/config', {
      method: 'POST',
      body: JSON.stringify(config),
    });
  };

  render() {
    return (
      <Context.Provider
        value={{
          loaded: this.loaded,
          connected: this.connected,
          update: this.update,
          error: this.error,
          ...this.state,
        }}
      >
        {this.props.children}
      </Context.Provider>
    );
  }
}

export { Context, Provider };
