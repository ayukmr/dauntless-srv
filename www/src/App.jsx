import { Component } from 'react';
import { Context } from './Provider';

import Header from './Header';
import Frames from './Frames';
import Detections from './Detections';
import ProcessingSettings from './ProcessingSettings';

class App extends Component {
  static contextType = Context;

  render() {
    return this.context.loaded()
      ? <>
        <Header />
        <Frames />
        <Detections />
      </>
      : <em>loading...</em>;
  }
}

export default App;
