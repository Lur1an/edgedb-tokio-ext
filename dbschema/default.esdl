module default {
  type User {
    required name: str;
    required age: int64 {
      constraint min_value(1);
    }
    org: Organization;
  }

  type Organization {
    required name: str;
    multi link users := .<org[is User];
  }
}
