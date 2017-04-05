import React from 'react';
import { connect } from 'dva';

function IndexPage() {
  return (
    <h1>{'osu!track homepage'}</h1>
  );
}

IndexPage.propTypes = {
};

export default connect()(IndexPage);
