# serde-xml

## current state
- [x] xml to struct deserialization
- [x] deserialize bool, int, string from `<anytagname>value</anythingelse>`
- [x] deserialize sequences (tuple, array, vector) as struct member
- [x] deserialize escaped chars (`&abcd;`)
- [x] deserialize CDATA
- [ ] deserialize enumerations
- [ ] deserialize arrays of enumerations
- [ ] deserialize errors instead of assertions
- [ ] more deserialize tests
- [x] parse to dom tree
- [ ] struct to xml serialization

## anti-features
I'm currently ignoring quite some stuff that I don't know how to handle in another way

- [x] ignore namespaces
- [x] skip xml comments
- [x] skip xml version tag
- [x] ignoring xml-attributes

## nice to have
- [ ] xsd verification
- [ ] sequences of sequences (how would these even look like in xml?)
- [ ] attributes to collapse xml elements that only contain a single type of element.
