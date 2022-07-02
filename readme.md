# Normalize OBJ

It's often the case that you want to normalize an OBJ file to the unit box `[-1, -1, -1],
[1,1,1]`, or move the average of its vertices to 0. This is a small tool for doing so.

It also comes with a utility to normalize one mesh to another's bounding box or center.

## Installation and Usage

```
git clone git@github.com:JulianKnodt/normalize_obj.git
cd normalize_obj
cargo install .
...
normalize-obj --src <SRC_OBJ_FILE> --output <OUTPUT_OBJ>.obj --method [centroid | aabb] \
  --target <REFERENCE_OBJ_FILE?>
```

## Contributing

If you have any other additional functionality you want added, please feel free to submit a pull
request!

