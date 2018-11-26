use game2d::pool::Pool;

/// Dummy object useful for pool tests
struct Person {
    name: &'static str,
    age: u32,
}

#[test]
fn can_add_and_remove_objects_into_pool() {
    let mut pool: Pool<Person> = Pool::new();
    assert_eq!(pool.len(), 0);
    assert_eq!(pool.is_empty(), true);
    assert_eq!(pool.capacity() > 0, true);

    let handle_joe = pool.push(Person {
        name: "Joe",
        age: 23,
    });
    assert_eq!(pool.len(), 1);
    assert_eq!(pool.is_empty(), false);

    let handle_jane = pool.push(Person {
        name: "Jane",
        age: 27,
    });
    assert_eq!(pool.len(), 2);

    let handle_pat = pool.push(Person {
        name: "Pat",
        age: 45,
    });
    assert_eq!(pool.len(), 3);

    assert_eq!(pool.get(handle_joe).unwrap().name, "Joe");
    assert_eq!(pool.get(handle_joe).unwrap().age, 23);
    assert_eq!(pool.get(handle_jane).unwrap().name, "Jane");
    assert_eq!(pool.get(handle_jane).unwrap().age, 27);
    assert_eq!(pool.get(handle_pat).unwrap().name, "Pat");
    assert_eq!(pool.get(handle_pat).unwrap().age, 45);

    let removed_jane = pool.remove(handle_jane);
    assert_eq!(removed_jane.unwrap().name, "Jane");
    assert_eq!(pool.len(), 2);

    assert_eq!(pool.remove(handle_jane).is_none(), true);
    assert_eq!(pool.get(handle_jane).is_none(), true);

    // Allocating after removal should re-use an existing allocation slot
    let handle_jack = pool.push(Person {
        name: "Jack",
        age: 35,
    });
    assert_eq!(pool.len(), 3);

    // Trying to remove / query via an old handle doesn't work
    assert_eq!(pool.remove(handle_jane).is_none(), true);
    assert_eq!(pool.get(handle_jane).is_none(), true);
    assert_eq!(pool.len(), 3);

    // Remove everyone else and make sure we can still add an object to an empty pool
    pool.remove(handle_pat);
    assert_eq!(pool.len(), 2);

    pool.remove(handle_joe);
    assert_eq!(pool.len(), 1);

    pool.remove(handle_jack);
    assert_eq!(pool.len(), 0);

    let handle_jill = pool.push(Person {
        name: "Jill",
        age: 35,
    });
    assert_eq!(pool.len(), 1);

    pool.remove(handle_jill);
    assert_eq!(pool.len(), 0);
    assert_eq!(pool.is_empty(), true);
}

#[test]
fn removing_by_handle_multiple_times_is_harmless() {
    let mut pool: Pool<&'static str> = Pool::new();
    let handle_lorem = pool.push("lorem");
    pool.push("ipsem");

    pool.remove(handle_lorem);
    pool.remove(handle_lorem);
    pool.remove(handle_lorem);

    pool.push("dolor");
    pool.push("sit");
    pool.push("amet");
}

#[test]
fn capacity_automatically_resizes() {
    let mut pool: Pool<i32> = Pool::with_capacity(3);

    assert_eq!(pool.capacity(), 3);

    pool.push(1);
    pool.push(2);
    pool.push(3);
    pool.push(4);

    assert_eq!(pool.capacity() > 3, true);
}

#[test]
#[should_panic(expected = "Can't create a pool with a capacity of 0")]
fn capacity_must_be_greater_than_zero() {
    Pool::<bool>::with_capacity(0);
}

#[test]
fn can_iterate_entries() {
    let mut pool: Pool<i32> = Pool::new();

    pool.push(9);
    pool.push(7);
    let mid_handle = pool.push(5);
    pool.push(3);
    pool.push(1);

    {
        let mut entries = pool.iter();
        assert_eq!(entries.next(), Some(&9));
        assert_eq!(entries.next(), Some(&7));
        assert_eq!(entries.next(), Some(&5));
        assert_eq!(entries.next(), Some(&3));
        assert_eq!(entries.next(), Some(&1));
        assert_eq!(entries.next(), None);
    }

    pool.remove(mid_handle);
    {
        // Entries can handle skipping a gap
        let mut entries = pool.iter();
        assert_eq!(entries.next(), Some(&9));
        assert_eq!(entries.next(), Some(&7));
        assert_eq!(entries.next(), Some(&3));
        assert_eq!(entries.next(), Some(&1));
        assert_eq!(entries.next(), None);
    }

    pool.push(100); // Should replace where 5 used to be
    {
        let mut entries = pool.iter();
        assert_eq!(entries.next(), Some(&9));
        assert_eq!(entries.next(), Some(&7));
        assert_eq!(entries.next(), Some(&100));
        assert_eq!(entries.next(), Some(&3));
        assert_eq!(entries.next(), Some(&1));
        assert_eq!(entries.next(), None);
    }
}

#[test]
fn can_iterate_entries_mutably() {
    let mut pool: Pool<i32> = Pool::new();

    pool.push(9);
    pool.push(7);
    let mid_handle = pool.push(5);
    pool.push(3);
    pool.push(1);

    pool.remove(mid_handle);

    for value in pool.iter_mut() {
        *value *= 10;
    }

    let mut entries = pool.iter();
    assert_eq!(entries.next(), Some(&90));
    assert_eq!(entries.next(), Some(&70));
    assert_eq!(entries.next(), Some(&30));
    assert_eq!(entries.next(), Some(&10));
    assert_eq!(entries.next(), None);
}

#[test]
fn can_iterate_handles() {
    let mut pool: Pool<i32> = Pool::new();

    pool.push(9);
    pool.push(7);
    let mid_handle = pool.push(5);
    pool.push(3);
    pool.push(1);

    pool.remove(mid_handle);

    for handle in pool.handles() {
        assert_eq!(pool.get(handle).is_some(), true);
        assert_eq!(pool.get_mut(handle).is_some(), true);

        *pool.get_mut(handle).unwrap() *= 10;
    }

    let mut entries = pool.iter();
    assert_eq!(entries.next(), Some(&90));
    assert_eq!(entries.next(), Some(&70));
    assert_eq!(entries.next(), Some(&30));
    assert_eq!(entries.next(), Some(&10));
    assert_eq!(entries.next(), None);
}
