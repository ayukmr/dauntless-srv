import { Component } from 'react';
import { Context } from './Provider';

const logo = new URL('assets/header.svg', import.meta.url);

class Header extends Component {
  static contextType = Context;

  render() {
    const connected = this.context.connected();

    return <header>
      <img src={logo} />

      <div style={connected ? { display: 'none' } : {}}>
        <h3>&middot;</h3>
        <h3>Disconnected</h3>
      </div>
    </header>;
  }
}

export default Header;
