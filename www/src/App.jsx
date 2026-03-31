import { Component } from 'react';
import { Context } from './Provider';

import Logo from './Logo';
import Frames from './Frames';
import Detections from './Detections';

class App extends Component {
  static contextType = Context;

  render() {
    const { id, meta, isLoaded, updateID } = this.context;

    return isLoaded()
      ? <>
        <header>
          <Logo />

          <div>
            {Array.from({ length: meta.n_cams }).map((_, i) => (
              <button
                style={i === id ? { borderColor: 'var(--fg)' } : {}}
                onClick={() => updateID(i)}
              >{i}</button>
            ))}
          </div>
        </header>

        <Frames />
        <Detections />
      </>
      : <em>loading...</em>;
  }
}

export default App;
