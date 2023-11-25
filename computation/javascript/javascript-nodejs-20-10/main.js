const express = require('express');
const bcrypt = require('bcryptjs');

const app = express();
const port = 3000;

app.get('/', (req, res) => {
    const password = req.query.password;
    const salt = req.query.salt;
    const hash = bcrypt.hashSync(password, salt);
    res.send(hash);
});

app.listen(port, () => {
    console.log(`Running on http://localhost:${port}`);
});
