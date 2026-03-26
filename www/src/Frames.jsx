import { Component } from 'react';
import { Context } from './Provider';

import { BlocksIcon, CameraIcon, SettingsIcon } from 'lucide-react';

import Frame from './Frame';
import CameraSettings from './CameraSettings';
import ProcessingSettings from './ProcessingSettings';

class Frames extends Component {
  static contextType = Context;

  state = {
    cameraSettings: false,
    processingSettings: false,
  };

  render() {
    const { data, meta, config } = this.context;
    const { cameraSettings, processingSettings } = this.state;

    return <section>
      <div>
        <h3 className="info">
          <span><CameraIcon /> {meta.cams[config.server.camera][0]}</span>
          <SettingsIcon
            style={{ cursor: 'pointer' }}
            onClick={() => this.setState({ cameraSettings: true })}
          />
        </h3>

        <Frame url="/api/frame" showIDs />

        <h3 className="info">
          <span style={{ fontVariantNumeric: 'tabular-nums' }}>
            {Math.trunc(1000 / data.ms)} FPS
          </span>
          <span>{config.server.res[0]}×{config.server.res[1]}</span>
        </h3>
      </div>

      <div>
        <h3 className="info">
          <span><BlocksIcon /> Mask</span>
          <SettingsIcon
            style={{ cursor: 'pointer' }}
            onClick={() => this.setState({ processingSettings: true })}
          />
        </h3>

        <Frame url="/api/mask" />
      </div>

      <CameraSettings
        show={cameraSettings}
        onCancel={() => this.setState({ cameraSettings: false })}
      />
      <ProcessingSettings
        show={processingSettings}
        onCancel={() => this.setState({ processingSettings: false })}
      />
    </section>;
  }
}

export default Frames;
