CREATE MIGRATION m1yany2z6xxma5aa2c3vskheuswzlytl5ujm7lvyx4w27wncxbl6kq
    ONTO initial
{
  CREATE TYPE default::Organization {
      CREATE REQUIRED PROPERTY name: std::str;
  };
  CREATE TYPE default::User {
      CREATE LINK org: default::Organization;
      CREATE REQUIRED PROPERTY age: std::int64 {
          CREATE CONSTRAINT std::min_value(1);
      };
      CREATE REQUIRED PROPERTY name: std::str;
  };
  ALTER TYPE default::Organization {
      CREATE MULTI LINK users := (.<org[IS default::User]);
  };
};
