<template>
  <div>Test complex nested HTML squashing</div>
</template>

<!-- Complex nested structure - all whitespace should collapse to single spaces -->
<metadata>
  <div>
    <ul>
      <li>
        <strong>Item    one    with    extra    spaces</strong>
      </li>
      <li>
        <em>Item    two</em>
      </li>
    </ul>
    <p>
      Paragraph    with
      multiple    lines
      and    spaces
    </p>
  </div>
</metadata>
