import { Component } from 'react';
import { Context } from './Provider';

import Tags from './Tags';
import { QrCodeIcon, SearchIcon } from 'lucide-react';

class Detections extends Component {
  static contextType = Context;

  render() {
    const { tags } = this.context.data;
    const valid = tags.filter((tag) => tag.id !== null);

    return <section>
      <div>
        <h3><QrCodeIcon /> Tags</h3>
        <Tags />
      </div>

      <div>
        <h3><SearchIcon /> Detections</h3>
        {valid.length === 0
          ? <i>{'<no tags>'}</i>
          : valid.map((tag, i) => {
            const { id, rot, pos } = tag;

            return <div key={`tag${i}`}>
              {i !== 0 && <hr />}

              <h4>Tag {id}</h4>
              <p>
                Rot: {rot.toFixed(2)}
                <br />
                X: {pos[0].toFixed(2)}, Y: {pos[1].toFixed(2)}, Z: {pos[2].toFixed(2)}
              </p>
            </div>;
          })}
      </div>
    </section>;
  }
}

export default Detections;
