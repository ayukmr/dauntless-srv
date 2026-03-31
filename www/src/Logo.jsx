import { Component } from 'react';
import { Context } from './Provider';

const logo = new URL('assets/header.svg', import.meta.url);

class Header extends Component {
  static contextType = Context;

  render() {
    const connected = this.context.isConnected();

    return <div id="logo">
      <img
        src={logo}
        style={connected ? {} : { filter: 'grayscale(100%)' }}
      />

      <div style={connected ? { display: 'none' } : {}}>
        <h3>&middot;</h3>
        <h3 style={{ marginLeft: 15, color: '#ed0000' }}>Disconnected</h3>
      </div>
    </div>;
  }
}

export default Header;
