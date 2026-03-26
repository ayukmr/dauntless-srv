import { Component } from 'react';

import { XIcon } from 'lucide-react';

class Popup extends Component {
  render() {
    return <div
      className="popup-bg"
      style={this.props.show ? {} : { display: 'none' }}
      onClick={() => this.props.onCancel()}
    >
      <div className="popup" onClick={(e) => e.stopPropagation()}>
        <div className="popup-header">
          <h2>{this.props.header}</h2>
          <XIcon
            style={{ cursor: 'pointer' }}
            onClick={() => this.props.onCancel()}
          />
        </div>

        {this.props.children}
      </div>
    </div>;
  }
}

export default Popup;
