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
    this.interval = setInterval(() => this.fetchData(), 50);
  }

  componentWillUnmount() {
    clearInterval(this.interval);
  }

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

      this.setConfig(updated);
      return { config: updated };
    });
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

  fetch = async (path, key) => {
    try {
      const res = await fetch(path);
      this.setState({ [key]: await res.json() });
      this.updateError(path, false);
    } catch {
      this.updateError(path, true);
    }
  };

  fetchData = async () => {
    await this.fetch(`/api/${this.state.id}/data`, 'data');
  };

  fetchMeta = async () => {
    await this.fetch('/api/meta', 'meta');
  };

  fetchConfig = async () => {
    await this.fetch(`/api/${this.state.id}/config`, 'config');
  };

  setConfig = async (config) => {
    await fetch(`/api/${this.state.id}/config`, {
      method: 'POST',
      body: JSON.stringify(config),
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
