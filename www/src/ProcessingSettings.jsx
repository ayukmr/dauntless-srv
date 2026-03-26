import { Component } from 'react';
import { Context } from './Provider';

import Popup from './Popup';

class ProcessingSettings extends Component {
  static contextType = Context;

  render() {
    const { config: { detector }, update } = this.context;

    return <Popup
      header="Processing"
      show={this.props.show}
      onCancel={this.props.onCancel}
    >
      <div>
        <label htmlFor="hystHigh">Hysteresis High</label>
        <input
          type="number"
          name="hystHigh"
          defaultValue={detector.hyst_high}
          onChange={(e) => update('detector', { hyst_high: +e.target.value })}
        />
      </div>

      <div>
        <label htmlFor="hystLow">Hysteresis Low</label>
        <input
          type="number"
          name="hystLow"
          defaultValue={detector.hyst_low}
          onChange={(e) => update('detector', { hyst_low: +e.target.value })}
        />
      </div>

      <div>
        <input
          type="checkbox"
          name="filterRatios"
          defaultChecked={detector.filter_ratios}
          onChange={(e) => update('detector', { filter_ratios: e.target.checked })}
        />
        <label htmlFor="filterRatios">Filter Ratios</label>
      </div>

      <div>
        <input
          type="checkbox"
          name="filterAngles"
          defaultChecked={detector.filter_angles}
          onChange={(e) => update('detector', { filter_angles: e.target.checked })}
        />
        <label htmlFor="filterAngles">Filter Angles</label>
      </div>
    </Popup>;
  }
}

export default ProcessingSettings;
